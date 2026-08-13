(async () => {
    const file = await fetch("/projects/synacor/challenge.bin");
    const buf = await file.arrayBuffer();
    const data = new DataView(buf);

    const terminal = document.getElementById("terminal");

    // memory contains the heap, with the registers appended to the end
    const memory = Array(0x7FFFF + 8).fill(0);
    const stack = [];

    for (let i = 0; i < data.byteLength; i += 2) {
        memory[i / 2] = data.getUint16(i, true);
    }

    // read from a memory index
    function read(idx) {
        return memory[idx % (2 ** 15)]
    }

    // get a number value
    function number(num) {
        if (num >= 2 ** 15) {
            return memory[num];
        } else {
            return num;
        }
    }

    function yieldToMain() {
        if (globalThis.scheduler?.yield) {
            return scheduler.yield();
        }

        // Fall back to yielding with setTimeout.
        return new Promise(resolve => {
            setTimeout(resolve, 0);
        });
    }

    async function* interpret() {
        let input = [];
        let ip = 0;

        let lastYield = performance.now();

        while (true) {
            if (performance.now() - lastYield > 50) {
                await yieldToMain();
                lastYield = performance.now();
            }

            let op = read(ip);
            switch (op) {
                // halt
                case 0:
                    ip += 1;
                    return;

                // set a b
                case 1:
                    memory[read(ip + 1)] = number(read(ip + 2));
                    ip += 3;
                    break;

                // push a
                case 2:
                    stack.push(number(read(ip + 1)));
                    ip += 2;
                    break;

                // pop a
                case 3:
                    const pop = stack.pop();
                    if (pop == undefined) {
                        console.error("Stack pop");
                    }
                    memory[read(ip + 1)] = pop;
                    ip += 2;
                    break;

                // eq a b c
                case 4:
                    if (number(read(ip + 2)) == number(read(ip + 3))) {
                        memory[read(ip + 1)] = 1;
                    } else {
                        memory[read(ip + 1)] = 0;
                    }
                    ip += 4;
                    break;

                // gt a b c
                case 5:
                    if (number(read(ip + 2)) > number(read(ip + 3))) {
                        memory[read(ip + 1)] = 1;
                    } else {
                        memory[read(ip + 1)] = 0;
                    }
                    ip += 4;
                    break;

                // jump a
                case 6:
                    ip = number(read(ip + 1));
                    break;

                // jt a b
                case 7:
                    if (number(read(ip + 1)) != 0) {
                        ip = number(read(ip + 2))
                    } else {
                        ip += 3;
                    }
                    break;

                // jf a b
                case 8:
                    if (number(read(ip + 1)) == 0) {
                        ip = number(read(ip + 2))
                    } else {
                        ip += 3;
                    }
                    break;

                // add a b c
                case 9:
                    memory[read(ip + 1)] = (number(read(ip + 2)) + number(read(ip + 3))) % (2 ** 15);
                    ip += 4;
                    break;

                // mult a b c
                case 10:
                    memory[read(ip + 1)] = (number(read(ip + 2)) * number(read(ip + 3))) % (2 ** 15);
                    ip += 4;
                    break;

                // mod a b c
                case 11:
                    memory[read(ip + 1)] = (number(read(ip + 2)) % number(read(ip + 3))) % (2 ** 15);
                    ip += 4;
                    break;

                // and a b c
                case 12:
                    memory[read(ip + 1)] = (number(read(ip + 2)) & number(read(ip + 3))) % (2 ** 15);
                    ip += 4;
                    break;

                // or a b c
                case 13:
                    memory[read(ip + 1)] = (number(read(ip + 2)) | number(read(ip + 3))) % (2 ** 15);
                    ip += 4;
                    break;

                // not a b
                case 14:
                    memory[read(ip + 1)] = (~number(read(ip + 2)) & 0x7FFF) % (2 ** 15);
                    ip += 3;
                    break;

                // rmem a b
                case 15:
                    memory[read(ip + 1)] = memory[number(read(ip + 2))];
                    ip += 3;
                    break;

                // wmem a b
                case 16:
                    memory[number(read(ip + 1))] = number(read(ip + 2));
                    ip += 3;
                    break;

                // call a
                case 17:
                    const a = number(read(ip + 1));
                    ip += 2;
                    stack.push(ip);
                    ip = a;
                    break;

                // ret
                case 18:
                    const ret_pop = stack.pop();
                    if (ret_pop == undefined) {
                        console.error("Stack ret");
                    }
                    ip = ret_pop;
                    break;

                // out a
                case 19:
                    const ch = String.fromCharCode(number(read(ip + 1)));
                    terminal.innerText += ch;
                    ip += 2;
                    break;

                // in a
                case 20:
                    if (input.length == 0) {
                        input = (yield).split('');
                    }
                    memory[read(ip + 1)] = input.shift().charCodeAt(0);
                    ip += 2;
                    break;

                // noop
                case 21:
                    ip += 1;
                    break;

                // error
                default:
                    console.error("Unknown instr: %i", op)
                    return;
            }
        }
    }

    const interp = interpret();
    interp.next();
    const input = `take tablet
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
look strange book`;

    const input_el = document.getElementById("input");
    const submit_el = document.getElementById("submit");

    for (const line of input.split('\n')) {
        await interp.next(line + '\n');
        input_el.focus();
        input_el.scrollIntoView();
    }

    submit_el.addEventListener("click", e => {
        e.preventDefault();
        const text = input_el.value;
        input_el.value = "";
        interp.next(text + "\n");
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
