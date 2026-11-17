function getRandomInt(min, max) {
    min = Math.ceil(Math.min(min, max));
    max = Math.floor(Math.max(min, max));
    return Math.floor(Math.random() * (max - min + 1)) + min;
}

function getRandomFloat(min, max) {
    min = Math.min(min, max);
    max = Math.max(min, max);
    return Math.random() * (max - min + 1) + min;
}

class BackgroundLayer {
    /** @type HTMLCanvasElement */ canvas;
    /** @type WebGLRenderingContext */ gl;

    vert = `
attribute vec4 aPos;
varying vec4 vertexColor;

void main() {
    gl_Position = aPos;
    vertexColor = vec4(aPos.x, aPos.y, 0.0, 1.0);
}`
    frag = `
// https://rhept.org/posts/scattering/
precision highp float;
varying vec4 vertexColor;
uniform vec2 resolution;
uniform float time;

#define camera_z 1.0

const float I0 = 11.0;
const vec3 sigma_r = vec3(0.33, 0.78, 1.89);
const float S_r = 0.17;
const float S_m = 0.05;
const float S_s = 0.07;
const float mie_w = 0.25;
const float sun_w = 0.0002;

vec3 sky_full(vec3 vd, vec3 sd) {
    float phase_r = 0.06 * (1.0 + dot(vd, sd) * dot(vd, sd));
    float phase_m = mie_w / (1.0 + mie_w + dot(vd, sd));
    phase_m = 0.3 * phase_m * phase_m;
    float phase_s = sun_w / (1.0 + sun_w + dot(vd, sd));
    phase_s = 4.0 * phase_s * phase_s;

    vec3 sigma_sum = S_r * sigma_r + S_m;
    vec3 phase_sum = S_r * sigma_r * phase_r + S_m * phase_m + S_s * phase_s;

    return I0 * sd.y / (sd.y + vd.y) * (phase_sum / sigma_sum) *
           (exp(sigma_sum / sd.y) - exp(-sigma_sum / vd.y));
}

void main() {
	vec2 uv = (2.*gl_FragCoord.xy - resolution.xy) / resolution.x;
    vec2 mouse_pos = 0.5 * vec2(1.5 * cos(time), 0.05 + 0.48 * (1.0 + sin(time)));

    vec3 view_dir = normalize(vec3(uv, camera_z));
    vec3 view_dir_reflected = vec3(view_dir.x, abs(view_dir.y), view_dir.z);
    vec3 sun_dir = -normalize(vec3(mouse_pos, camera_z));

    vec3 col = sky_full(view_dir_reflected, sun_dir);

    if(view_dir.y < 0.0) {
        col = vec3(30.0 / 256.0);
    }
    col = pow(col, vec3(1.0 / 2.2)); // gamma correct

    gl_FragColor = vec4(col, 1.0);
}`

    constructor(/** @type HTMLCanvasElement */ canvas) {
        this.canvas = canvas;
        const gl = this.canvas.getContext("webgl", {
            alpha: false,
            depth: false,
            stencil: false,
            antialias: true,
            preserveDrawingBuffer: false,
        });
        if (gl) {
            this.gl = gl
        } else {
            throw "WebGL support unavaliable";
        }

        const vs = this.loadShader(gl.VERTEX_SHADER, this.vert);
        const fs = this.loadShader(gl.FRAGMENT_SHADER, this.frag);
        const program = gl.createProgram();
        gl.attachShader(program, vs);
        gl.attachShader(program, fs);
        gl.linkProgram(program);
        if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
            const log = gl.getProgramInfoLog(program);
            throw `Unable to initialize the shader program: ${log}`;
        }

        this.position_loc = gl.getAttribLocation(program, "aPos");
        this.position_buf = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, this.position_buf);
        const positions = [-1, -1, 3, -1, -1, 3];
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

        this.resolution_loc = gl.getUniformLocation(program, "resolution");
        this.time_loc = gl.getUniformLocation(program, "time");

        this.program = program;
    }

    loadShader(type, source) {
        const gl = this.gl;

        const shader = gl.createShader(type);
        gl.shaderSource(shader, source);
        gl.compileShader(shader);

        if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
            const log = gl.getShaderInfoLog(shader);
            gl.deleteShader(shader);
            throw `An error occurred compiling the shaders: ${log}`;
        }
        return shader;
    }

    frame({ time, size }) {
        const gl = this.gl;

        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.useProgram(this.program);
        gl.enableVertexAttribArray(this.position_loc);
        gl.bindBuffer(gl.ARRAY_BUFFER, this.position_buf);
        gl.vertexAttribPointer(this.position_loc, 2, gl.FLOAT, false, 0, 0);
        gl.uniform2f(this.resolution_loc, size.canvas_width, size.canvas_height);
        gl.uniform1f(this.time_loc, time.elapsed / 10000);
        gl.drawArrays(gl.TRIANGLES, 0, 3);
    }

    resize({ size }) {
        this.canvas.width = size.canvas_width;
        this.canvas.height = size.canvas_height;
        this.gl.viewport(0, 0, size.canvas_width, size.canvas_height);
    }
}

class BuildingLayer {
    /** @type HTMLCanvasElement */ canvas;
    buildings = [];
    size = "";

    constructor(/** @type HTMLCanvasElement */ canvas) {
        this.canvas = canvas;
        this.rough = rough.canvas(canvas);

        for (let i = 0; i < 15; i++) {
            const scale = Math.random() / 2;
            let b = new Building();

            this.buildings.push({
                x: Math.random(),
                z: getRandomFloat(0, 40),
                building: b,
            });
        }

        this.buildings.sort((a, b) => b.z - a.z);
    }

    frame({ size }) {
        if (JSON.stringify(size) == this.size) {
            return;
        }
        this.size = JSON.stringify(size);

        this.rough.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        for (const building of this.buildings) {
            building.building.draw(
                size,
                size.frame_width * building.x,
                size.frame_height * building.z / 100,
                (100 - building.z) / 100,
                this.rough,
            );
        }
    }

    resize({ size }) {
        this.canvas.width = size.canvas_width;
        this.canvas.height = size.canvas_height;
    }
}

class Building {
    static gen = rough.generator();

    constructor() {
        const height = getRandomInt(2, 10);
        const widths = [2, 2, 2, 3, 3, 4];
        const width = widths[getRandomInt(0, widths.length - 1)];
        const floor_height = getRandomInt(70, 80);
        const column_width = getRandomInt(80, 110);

        const door_idx = getRandomInt(0, width - 1);
        const door_width = getRandomInt(14, 18);
        const door_height = getRandomInt(38, 42);

        const window_height = getRandomInt(16, 20);
        const window_width = getRandomInt(22, 28);

        let rects = [{
            x: 0,
            y: 0,
            width: width * column_width,
            height: height * floor_height,
        }];

        for (let i = 0; i < width; i++) {
            if (i == door_idx) {
                rects.push({
                    x: i * column_width + (column_width - door_width) / 2,
                    y: 0,
                    width: door_width,
                    height: door_height,
                })
            } else {
                rects.push({
                    x: i * column_width + (column_width - window_width) / 2,
                    y: door_height - window_height,
                    width: window_width,
                    height: window_height,
                })
            }
        }

        for (let y = 1; y < height; y++) {
            for (let x = 0; x < width; x++) {
                if (Math.random() > 0.98) {
                    continue;
                }
                rects.push({
                    x: x * column_width + (column_width - window_width) / 2,
                    y: y * floor_height + (door_height - window_height),
                    width: window_width,
                    height: window_height,
                })
            }
        }

        for (const rect of rects) {
            rect.fill = `rgba(${Math.random() * 256}, ${Math.random() * 256}, ${Math.random() * 256}, 1.0)`;
            rect.seed = Math.ceil(Math.random() * (2 ** 31));
            rect.roughness = getRandomFloat(0.5, 1.2);
        }

        this.rects = rects;
    }

    draw(size, x, y, scale, ctx) {
        for (const rect of this.rects) {
            ctx.rectangle(
                rect.x * size.frame_scale * scale + size.horizontal_offset + x * size.frame_scale,
                size.canvas_height - ((rect.y + rect.height) * size.frame_scale * scale + size.vertical_offset + y * size.frame_scale),
                rect.width * size.frame_scale * scale,
                rect.height * size.frame_scale * scale,
                { fill: rect.fill, fillStyle: `solid`, seed: rect.seed, roughness: rect.roughness }
            );
        }
    }
}

class Cityscape {
    layers = [
        new BackgroundLayer(document.getElementById("background")),
        new BuildingLayer(document.getElementById("buildings")),
    ];

    time = {
        start: performance.now(),
        elapsed: 0,
        delta: 0,
    };

    size = {
        // device pixel size of a canvas
        canvas_height: 0,
        canvas_width: 0,
        device_pixel_ratio: 0,

        // size of a 1920*1080 landscape rectangle within the canvas
        frame_width: 1920,
        frame_height: 1080,
        frame_pixel_width: 0,
        frame_pixel_height: 0,
        // we don't care about device pixel ratio, just fitting the rectangle
        // onto the screen somehow, and making it not-blurry, i'm aware this will
        // likely look awful on small phones, but hopefully high dpi large monitors
        // work well.
        frame_scale: 0,

        // offsets to position that rectangle
        vertical_offset: 0,
        horizontal_offset: 0,
    };

    // Run a single frame's rendering for all layers
    frame() {
        let now = performance.now();
        let new_elapsed = now - this.time.start;
        this.time.delta = new_elapsed - this.time.elapsed;
        this.time.elapsed = new_elapsed;

        let data = { time: this.time, size: this.size };
        for (const layer of this.layers) {
            layer.frame(data)
        }
    }

    // Run resize logic for all layers
    // See link below for resize device pixel ratio logic:
    // https://stackoverflow.com/questions/74616600/html-canvas-coordinate-systems-and-rendering-process
    resize(/** @type ResizeObserverEntry */ entry) {
        let width;
        let height;
        const dPR = window.devicePixelRatio;
        if (entry.devicePixelContentBoxSize) {
            width = entry.devicePixelContentBoxSize[0].inlineSize;
            height = entry.devicePixelContentBoxSize[0].blockSize;
        } else if (entry.contentBoxSize) {
            if (entry.contentBoxSize[0]) {
                width = entry.contentBoxSize[0].inlineSize * dPR;
                height = entry.contentBoxSize[0].blockSize * dPR;
            } else {
                width = entry.contentBoxSize.inlineSize * dPR;
                height = entry.contentBoxSize.blockSize * dPR;
            }
        } else {
            width = entry.contentRect.width * dPR;
            height = entry.contentRect.height * dPR;
        }
        this.size.canvas_width = width;
        this.size.canvas_height = height;
        this.size.device_pixel_ratio = dPR;

        let fPW = this.size.canvas_width;
        let fPH = Math.floor(fPW / this.size.frame_width * this.size.frame_height);
        if (fPH > this.size.canvas_height) {
            fPH = this.size.canvas_height;
            fPW = fPH / this.size.frame_height * this.size.frame_width;
        }
        this.size.frame_pixel_height = fPH;
        this.size.frame_pixel_width = fPW;
        this.size.frame_scale = this.size.frame_pixel_height / this.size.frame_height;

        this.size.vertical_offset = (this.size.canvas_height - this.size.frame_pixel_height) / 2;
        this.size.horizontal_offset = (this.size.canvas_width - this.size.frame_pixel_width) / 2;

        let data = { size: this.size };
        for (const layer of this.layers) {
            layer.resize(data)
        }
        this.frame();
    }

    // Start the animation loop
    run() {
        let observer = new ResizeObserver(([entry]) => {
            this.resize(entry);
        });
        observer.observe(document.querySelector("canvas"), { box: "device-pixel-content-box" });

        let callback = () => {
            this.frame();
            requestAnimationFrame(callback);
        };
        requestAnimationFrame(callback);
    }
}

const city = new Cityscape();
city.run();
