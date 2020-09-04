import assert from './assert.js';

const H_SCALE = Math.sqrt(3) * 0.5;

let canvas: HTMLCanvasElement;
let ctx: CanvasRenderingContext2D;

function hex_path(x: number, y: number, size: number) {
    ctx.beginPath();
    let w = H_SCALE * size * 0.5;
    let s = size * 0.25;
    ctx.moveTo(x - w, y - s);
    ctx.lineTo(x - w, y + s);
    ctx.lineTo(x, y + 2 * s);
    ctx.lineTo(x + w, y + s);
    ctx.lineTo(x + w, y - s);
    ctx.lineTo(x, y - 2 * s);
    ctx.closePath();
}

function draw_ant(
    { x, y, dir, color, has_food, size }:
    { x: number; y: number; dir: number; color: string; has_food: boolean; size: number; }
) {
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(size, size);
    ctx.rotate(dir * Math.PI / 3);

    if (has_food) {
        ctx.fillStyle = '#0c0';
        hex_path(0.2, 0, 0.2);
        ctx.fill();
    }

    ctx.fillStyle = color;
    if (size > 15) {
        ctx.beginPath();
        ctx.ellipse(-0.2, 0, 0.13, 0.09, 0, 0, 2 * Math.PI);
        ctx.ellipse(0, 0, 0.07, 0.05, 0, 0, 2 * Math.PI);
        ctx.ellipse(0.12, 0, 0.05, 0.07, 0, 0, 2 * Math.PI);
        ctx.fill();

        ctx.lineWidth = 0.02;
        ctx.strokeStyle = color;
        ctx.beginPath();
        for (let sign = -1; sign <= 1; sign += 2) {
            ctx.moveTo(0.0, 0);
            ctx.lineTo(-0.2, 0.2 * sign);
            ctx.moveTo(0.03, 0);
            ctx.lineTo(-0.1, 0.2 * sign);
            ctx.moveTo(0, 0);
            ctx.lineTo(0.15, 0.15 * sign);
        }
        ctx.stroke();
    } else {
        ctx.fillRect(-0.3, -0.1, 0.5, 0.2);
    }

    ctx.restore();
}

function draw(offset_x: number, offset_y: number, scale: number, lines: string[]) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    ctx.fillStyle = 'gray';
    ctx.strokeStyle = 'black';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    lines.forEach((line, i) => {
        let y = (i + 0.666) * 0.75 * scale + offset_y;
        if (y + scale < 0 || y - scale > canvas.height) {
            return;
        }
        Array.from(line).forEach((c, j) => {
            let x = (j + 1) * 0.5 * H_SCALE * scale + offset_x;
            if (x + scale < 0 || x - scale > canvas.width) {
                return;
            }
            if (c == '#') {
                ctx.fillStyle = 'black';
                hex_path(x, y, scale);
                ctx.fill();
            } else if (c == '+' || c == '-') {
                ctx.strokeStyle = c == '+' ? 'rgba(255, 0, 0, 0.3)' : 'rgba(0, 0, 255, 0.3)';
                let color = c == '+' ? 'red' : 'blue';
                hex_path(x, y, 0.9 * scale);
                ctx.stroke();
                draw_ant({ x, y, dir: 0, color, has_food: i % 2 == 0 && j / 2 % 2 == 0, size: scale });
            }
            else if (/\d/.test(c)) {
                ctx.fillStyle = '#0f0';
                hex_path(x, y, Math.sqrt(parseInt(c) / 10) * scale);
                ctx.fill();
                if (scale >= 15) {
                    ctx.fillStyle = 'black';
                    ctx.fillText(c, x, y);
                }
            }
        });
    });
}

async function main() {
    // let resp = await fetch('/data/tiny.world');
    let resp = await fetch('/data/sample0.world');
    assert(resp.ok);
    let text = await resp.text();
    let lines = text.trimEnd().split('\n');
    let width = parseInt(lines[0]);
    let height = parseInt(lines[1]);
    lines = lines.slice(2);
    assert(lines.length == height);

    canvas = document.getElementById('canvas') as HTMLCanvasElement;
    ctx = canvas.getContext('2d')!;

    let hor_size = H_SCALE * (width + 0.5);
    let ver_size = height * 0.75 + 0.25;

    let offset_x = 0;
    let offset_y = 0;
    let scale = Math.min(canvas.width / hor_size, canvas.height / ver_size);
    draw(offset_x, offset_y, scale, lines);

    canvas.onpointerdown = (e) => canvas.setPointerCapture(e.pointerId);
    canvas.onpointerup = (e) => canvas.releasePointerCapture(e.pointerId);

    canvas.onwheel = (e) => {
        let delta: number;
        switch (e.deltaMode) {
            case e.DOM_DELTA_PIXEL:
                delta = e.deltaY;
                break;
            case e.DOM_DELTA_LINE:
                delta = e.deltaY * 12;
                break;
            default:
                assert(false);
        }
        let r = canvas.getBoundingClientRect();
        let x = e.clientX - r.left;
        let y = e.clientY - r.top;

        x -= offset_x;
        y -= offset_y;
        x /= scale;
        y /= scale;

        let old_scale = scale;
        scale *= Math.exp(-delta * 0.002);

        offset_x += x * (old_scale - scale);
        offset_y += y * (old_scale - scale);

        requestAnimationFrame(() => draw(offset_x, offset_y, scale, lines));
        e.preventDefault();
    };

    canvas.onpointermove = (e) => {
        if (e.buttons == 1) {
            offset_x += e.movementX;
            offset_y += e.movementY;
            requestAnimationFrame(() => draw(offset_x, offset_y, scale, lines));
        }
    }
}

main()
