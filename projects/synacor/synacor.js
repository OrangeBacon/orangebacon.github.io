'use strict';

// -----------------
// Utilities
// -----------------

// awaitable yield for long-running tasks
function yieldToMain() {
    if (globalThis.scheduler?.yield) {
        return scheduler.yield();
    }

    // Fall back to yielding with setTimeout.
    return new Promise(resolve => {
        setTimeout(resolve, 0);
    });
}

// -----------------
// Global UI
// -----------------
(() => {
    const tabs = [...document.querySelectorAll("body section")];
    const buttons = [...document.querySelectorAll("nav button")];
    let current = 0;

    for (let i = 0; i < buttons.length; i++) {
        const button = buttons[i];
        button.addEventListener('keydown', onKeydown);
        button.addEventListener('click', onClick);
        if (button.classList.contains("open")) {
            current = i;
        }
    }

    setTab(current, false);

    function onKeydown(e) {
        let target = event.currentTarget;
        let handled = false;

        switch (event.key) {
            case 'ArrowLeft':
                setTab(current - 1, true);
                handled = true;
                break;

            case 'ArrowRight':
                setTab(current + 1, true);
                handled = true;
                break;

            case 'Home':
                setTab(0, true);
                handled = true;
                break;

            case 'End':
                setTab(tabs.length - 1, true);
                handled = true;
                break;

            default:
                break;
        }

        if (handled) {
            e.stopPropagation();
            e.preventDefault();
        }
    }

    function onClick(e) {
        setTab(e.currentTarget, true)
    }

    function setTab(set, focus) {
        const len = buttons.length;

        let test = set;
        if (typeof test == "number") {
            // wrap to within the number of buttons
            test = ((set % len) + len) % len;
        }

        for (let i = 0; i < len; i++) {
            const button = buttons[i];
            const tab = tabs[i];

            if (button == test || i == test) {
                button.classList.add("open");
                tab?.classList.add("open");
                current = i;
                if (focus) {
                    button.focus();
                }
            } else {
                button.classList.remove("open");
                tab?.classList.remove("open");
            }
        }
    }
})();

// -----------------
// Interpreter
// -----------------
class Interpreter {
    memory = new Uint16Array(2 ** 15);
    registers = new Uint16Array(8);
    stack = [];
    ip = 0;

    // create a blank intepreter
    constructor() { }

    // load a binary source file into the interpreter
    load_file(binary) {
        const data = new DataView(binary);

        for (let i = 0; i < data.byteLength; i += 2) {
            this.memory[i / 2] = data.getUint16(i, true);
        }
    }

    #input = [];
    // add input text to the interpreter
    set input(value) {
        let inp = value.split('');
        inp.reverse();
        this.#input = this.#input.concat(inp);
    }

    #output = [];
    // get any new output text since last time it was checked
    get output() {
        let out = this.#output.join('');
        this.#output = [];
        return out;
    }

    // read from a memory index
    #read(idx) {
        return this.memory[idx % (2 ** 15)]
    }

    // get a number or a register value
    #number(num) {
        if (num >= 2 ** 15) {
            return this.registers[num - (2 ** 15)];
        } else {
            return num;
        }
    }

    // write to memory or register
    #write(addr, value) {
        if (addr >= 2 ** 15) {
            this.registers[addr - (2 ** 15)] = value % (2 ** 15);
        } else {
            this.memory[addr] = value % (2 ** 15);
        }
    }

    *interpret() {
        let lastYield = performance.now();

        while (true) {
            if (performance.now() - lastYield > 50) {
                yield "loop";
                lastYield = performance.now();
            }

            let op = this.#read(this.ip);
            switch (op) {
                // halt
                case 0:
                    this.ip += 1;
                    yield "halt";
                    break;

                // set a b
                case 1:
                    this.#write(this.#read(this.ip + 1), this.#number(this.#read(this.ip + 2)));
                    this.ip += 3;
                    break;

                // push a
                case 2:
                    this.stack.push(this.#number(this.#read(this.ip + 1)));
                    this.ip += 2;
                    break;

                // pop a
                case 3:
                    const pop = this.stack.pop();
                    if (pop == undefined) {
                        yield "stack_pop";
                        break;
                    }
                    this.#write(this.#read(this.ip + 1), pop);
                    this.ip += 2;
                    break;

                // eq a b c
                case 4:
                    if (this.#number(this.#read(this.ip + 2)) == this.#number(this.#read(this.ip + 3))) {
                        this.#write(this.#read(this.ip + 1), 1);
                    } else {
                        this.#write(this.#read(this.ip + 1), 0);
                    }
                    this.ip += 4;
                    break;

                // gt a b c
                case 5:
                    if (this.#number(this.#read(this.ip + 2)) > this.#number(this.#read(this.ip + 3))) {
                        this.#write(this.#read(this.ip + 1), 1);
                    } else {
                        this.#write(this.#read(this.ip + 1), 0);
                    }
                    this.ip += 4;
                    break;

                // jump a
                case 6:
                    this.ip = this.#number(this.#read(this.ip + 1));
                    break;

                // jt a b
                case 7:
                    if (this.#number(this.#read(this.ip + 1)) != 0) {
                        this.ip = this.#number(this.#read(this.ip + 2))
                    } else {
                        this.ip += 3;
                    }
                    break;

                // jf a b
                case 8:
                    if (this.#number(this.#read(this.ip + 1)) == 0) {
                        this.ip = this.#number(this.#read(this.ip + 2))
                    } else {
                        this.ip += 3;
                    }
                    break;

                // add a b c
                case 9:
                    this.#write(this.#read(this.ip + 1), this.#number(this.#read(this.ip + 2)) + this.#number(this.#read(this.ip + 3)));
                    this.ip += 4;
                    break;

                // mult a b c
                case 10:
                    this.#write(this.#read(this.ip + 1), this.#number(this.#read(this.ip + 2)) * this.#number(this.#read(this.ip + 3)));
                    this.ip += 4;
                    break;

                // mod a b c
                case 11:
                    this.#write(this.#read(this.ip + 1), this.#number(this.#read(this.ip + 2)) % this.#number(this.#read(this.ip + 3)));
                    this.ip += 4;
                    break;

                // and a b c
                case 12:
                    this.#write(this.#read(this.ip + 1), this.#number(this.#read(this.ip + 2)) & this.#number(this.#read(this.ip + 3)));
                    this.ip += 4;
                    break;

                // or a b c
                case 13:
                    this.#write(this.#read(this.ip + 1), this.#number(this.#read(this.ip + 2)) | this.#number(this.#read(this.ip + 3)));
                    this.ip += 4;
                    break;

                // not a b
                case 14:
                    this.#write(this.#read(this.ip + 1), ~this.#number(this.#read(this.ip + 2)) & 0x7FFF);
                    this.ip += 3;
                    break;

                // rmem a b
                case 15:
                    this.#write(this.#read(this.ip + 1), this.#read(this.#number(this.#read(this.ip + 2))));
                    this.ip += 3;
                    break;

                // wmem a b
                case 16:
                    this.#write(this.#number(this.#read(this.ip + 1)), this.#number(this.#read(this.ip + 2)));
                    this.ip += 3;
                    break;

                // call a
                case 17:
                    const a = this.#number(this.#read(this.ip + 1));
                    this.ip += 2;
                    this.stack.push(this.ip);
                    this.ip = a;
                    break;

                // ret
                case 18:
                    const ret_pop = this.stack.pop();
                    if (ret_pop == undefined) {
                        yield "stack_ret";
                        break;
                    }
                    this.ip = ret_pop;
                    break;

                // out a
                case 19:
                    const ch = String.fromCharCode(this.#number(this.#read(this.ip + 1)));
                    this.#output.push(ch);
                    if (ch == "\n") {
                        yield "line";
                    }
                    this.ip += 2;
                    break;

                // in a
                case 20:
                    if (this.#input.length == 0) {
                        yield "input";
                        break;
                    }
                    this.#write(this.#read(this.ip + 1), this.#input.pop().charCodeAt(0));
                    this.ip += 2;
                    break;

                // noop
                case 21:
                    this.ip += 1;
                    break;

                // error
                default:
                    yield "unknown";
                    break;
            }
        }
    }

    disassemble(idx) {
        let disassembly = [];

        while (idx < this.memory.length) {
            let op = this.#read(idx);

            const ops = [
                ["halt", 0],
                ["set", 2],
                ["push", 1],
                ["pop", 1],
                ["eq", 3],
                ["gt", 3],
                ["jmp", 1],
                ["jt", 2],
                ["jf", 2],
                ["add", 3],
                ["mult", 3],
                ["mod", 3],
                ["and", 3],
                ["or", 3],
                ["not", 2],
                ["rmem", 2],
                ["wmem", 2],
                ["call", 1],
                ["ret", 0],
                ["out", 1],
                ["in", 1],
                ["noop", 0],
            ];

            const decoded = ops[op];
            const last = disassembly[disassembly.length - 1];
            if (last?.startsWith("out") && decoded?.[0] == "out") {
                let arg = this.#read(idx + 1);
                disassembly[disassembly.length - 1] = last.concat(String.fromCharCode(arg));
                idx += 2;
            } else if (last?.startsWith("equ") && !decoded) {
                disassembly[disassembly.length - 1] = last.concat(String.fromCharCode(op));
                idx += 1;
            } else if (decoded) {
                let args = [decoded[0]];
                for (let i = 0; i < decoded[1]; i++) {
                    const val = this.#read(idx + i + 1);
                    if (val >= 2 ** 15) {
                        args.push(`r${val - (2 ** 15)}`);
                    } else if (decoded[0] == "out") {
                        args.push(String.fromCharCode(val));
                    } else {
                        args.push(val.toString());
                    }
                }

                disassembly.push(args.join(' '));
                idx += decoded[1] + 1;

            } else {
                disassembly.push(`equ ${String.fromCharCode(op)}`);
                idx += 1;
            }
        }

        return disassembly.join('\n');
    }
}

(async () => {
    const file = await fetch("/projects/synacor/challenge.bin");
    const buf = await file.arrayBuffer();

    const interp = new Interpreter();
    interp.load_file(buf);
    interp.input = `take tablet
use tablet
go doorway
go north
go north
go bridge
go continue
go down
go east
take empty lantern
go west
go west
go passage
go ladder
go west
go south
go north
take can
use can
go west
use lantern
go ladder
go darkness
go continue
go west
go west
go west
go west
go north
take red coin
go north
go east
take concave coin
go down
take corroded coin
go up
go west
go west
take blue coin
go up
take shiny coin
go down
go east
use blue coin
use red coin
use shiny coin
use concave coin
use corroded coin
go north
take teleporter
use teleporter
look strange book\n`;

    const input_el = document.getElementById("term_input");
    const submit_el = document.getElementById("submit");
    const terminal = document.getElementById("terminal");
    const disassembly = document.getElementById("disassembly");

    const run = interp.interpret();
    async function do_output() {
        while (true) {
            const res = run.next();
            if (res.done) {
                return;
            }
            switch (res.value) {
                case "input":
                case "halt": return;
                case "line":
                    terminal.appendChild(document.createTextNode(interp.output));
                    break;
                case "loop":
                    await yieldToMain();
                    break;
                case "stack_pop":
                case "stack_ret":
                case "unknown":
                    console.error(kind, interp);
                    break
            }
        }
    }
    await do_output();

    disassembly.appendChild(document.createTextNode(interp.disassemble(0)));

    submit_el.addEventListener("click", async e => {
        e.preventDefault();
        const text = input_el.value;
        input_el.value = "";
        interp.input = `${text}\n`;
        await do_output();
        input_el.scrollIntoView();
    });
    input_el.addEventListener("keydown", e => {
        if (e.key == "Enter") {
            e.preventDefault();
            submit_el.click();
        }
    });

    console.log("Done")
})()


