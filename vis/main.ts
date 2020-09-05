import assert from './assert.js';
import { Match, Background, ReplayFrame } from './types.js';

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

function transform(offset_x: number, offset_y: number, scale: number, col: number, row: number): {x: number, y: number} {
    let x = (col * 2 + row % 2 + 1) * 0.5 * H_SCALE * scale + offset_x;
    let y = (row + 0.666) * 0.75 * scale + offset_y;
    return {x, y};
}

function draw_background(offset_x: number, offset_y: number, scale: number, bg: Background) {
    ctx.fillStyle = 'black';
    bg.rocks.forEach(([j, i]) => {
        let {x, y} = transform(offset_x, offset_y, scale, j, i);
        hex_path(x, y, scale);
        ctx.fill();
    });
    ctx.strokeStyle = 'rgba(255, 0, 0, 0.3)';
    bg.red_anthill.forEach(([j, i]) => {
        let {x, y} = transform(offset_x, offset_y, scale, j, i);
        hex_path(x, y, 0.9 * scale);
        ctx.stroke();
    });
    ctx.strokeStyle = 'rgba(0, 0, 255, 0.3)';
    bg.black_anthill.forEach(([j, i]) => {
        let {x, y} = transform(offset_x, offset_y, scale, j, i);
        hex_path(x, y, 0.9 * scale);
        ctx.stroke();
    });
}

function draw_frame(offset_x: number, offset_y: number, scale: number, frame: ReplayFrame) {
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    frame.food.forEach(([j, i, amount]) => {
        let {x, y} = transform(offset_x, offset_y, scale, j, i);
        ctx.fillStyle = '#0f0';
        hex_path(x, y, Math.sqrt(amount / 10) * scale);
        ctx.fill();
        if (scale >= 15) {
            ctx.fillStyle = 'black';
            ctx.fillText('' + amount, x, y);
        }
    })

    frame.ants.forEach((ant) => {
        let {x, y} = transform(offset_x, offset_y, scale, ant.x, ant.y);
        let color = ant.color == 'red' ? 'red' : 'blue';
        draw_ant({ x, y, dir: ant.dir, color, has_food: ant.has_food, size: scale });
    });
}

async function main() {
    let { hash } = window.location;
    assert(hash.startsWith('#'), hash);
    hash = hash.slice(1);
    let match = JSON.parse(decodeURIComponent(hash)) as Match;
    let r = await fetch('/background?match=' + encodeURIComponent(JSON.stringify(match)));
    assert(r.ok);
    let bg = await r.json() as Background;
    
    r = await fetch('/frame?match=' + encodeURIComponent(JSON.stringify(match)) + '&frame_no=0');
    assert(r.ok);
    let frame = await r.json() as ReplayFrame;

    let draw_stuff = (offset_x: number, offset_y: number, scale: number) => {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        draw_background(offset_x, offset_y, scale, bg);
        draw_frame(offset_x, offset_y, scale, frame);
    };

    let max_x = Math.max(...bg.rocks.map(([x, y]) => x));
    let max_y = Math.max(...bg.rocks.map(([x, y]) => y));

    canvas = document.getElementById('canvas') as HTMLCanvasElement;
    ctx = canvas.getContext('2d')!;

    let hor_size = H_SCALE * (max_x + 1 + 0.5);
    let ver_size = (max_y + 1) * 0.75 + 0.25;

    let offset_x = 0;
    let offset_y = 0;
    let scale = Math.min(canvas.width / hor_size, canvas.height / ver_size);
    draw_stuff(offset_x, offset_y, scale);

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

        requestAnimationFrame(() => draw_stuff(offset_x, offset_y, scale));
        e.preventDefault();
    };

    canvas.onpointermove = (e) => {
        if (e.buttons == 1) {
            offset_x += e.movementX;
            offset_y += e.movementY;
            requestAnimationFrame(() => draw_stuff(offset_x, offset_y, scale));
        }
    }
}

main()
