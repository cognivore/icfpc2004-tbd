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

interface Transform {
    offset_x: number,
    offset_y: number,
    scale: number,
}

function transform(offset_x: number, offset_y: number, scale: number, col: number, row: number): {x: number, y: number} {
    let x = (col * 2 + row % 2 + 1) * 0.5 * H_SCALE * scale + offset_x;
    let y = (row + 0.666) * 0.75 * scale + offset_y;
    return {x, y};
}

function zoom(tr: Transform, centex_x: number, center_y: number, factor: number) {
    let x = (centex_x - tr.offset_x) / tr.scale;
    let y = (center_y - tr.offset_y) / tr.scale;

    let old_scale = tr.scale;
    tr.scale *= factor;

    tr.offset_x += x * (old_scale - tr.scale);
    tr.offset_y += y * (old_scale - tr.scale);

}

function apply_transform(tr: Transform, col: number, row: number): {x: number, y: number} {
    let x = (col * 2 + row % 2 + 1) * 0.5 * H_SCALE * tr.scale + tr.offset_x;
    let y = (row + 0.666) * 0.75 * tr.scale + tr.offset_y;
    return {x, y};
}

function unapply_transform(tr: Transform, x: number, y: number): {row: number, col: number} {
    let row = Math.round((y - tr.offset_y) / (tr.scale * 0.75) - 0.666);
    let col = Math.round(((x - tr.offset_x) / (0.5 * H_SCALE * tr.scale) - 1 - row % 2) / 2);
    return { row, col };
}

function draw_background(tr: Transform, bg: Background) {
    ctx.fillStyle = 'black';
    bg.rocks.forEach(([j, i]) => {
        let {x, y} = apply_transform(tr, j, i);
        hex_path(x, y, tr.scale);
        ctx.fill();
    });
    ctx.lineWidth = 1;
    ctx.strokeStyle = 'rgba(255, 0, 0, 0.3)';
    bg.red_anthill.forEach(([j, i]) => {
        let {x, y} = apply_transform(tr, j, i);
        hex_path(x, y, 0.9 * tr.scale);
        ctx.stroke();
    });
    ctx.strokeStyle = 'rgba(0, 0, 255, 0.3)';
    bg.black_anthill.forEach(([j, i]) => {
        let {x, y} = apply_transform(tr, j, i);
        hex_path(x, y, 0.9 * tr.scale);
        ctx.stroke();
    });
}

function draw_frame(tr: Transform, frame: ReplayFrame) {
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    frame.food.forEach(([j, i, amount]) => {
        let {x, y} = apply_transform(tr, j, i);
        ctx.fillStyle = '#0f0';
        hex_path(x, y, Math.sqrt(amount / 10) * tr.scale);
        ctx.fill();
        if (tr.scale >= 15) {
            ctx.fillStyle = 'black';
            ctx.fillText('' + amount, x, y);
        }
    })

    frame.ants.forEach((ant) => {
        let {x, y} = apply_transform(tr, ant.x, ant.y);
        let color = ant.color == 'red' ? 'red' : 'blue';
        draw_ant({ x, y, dir: ant.dir, color, has_food: ant.has_food, size: tr.scale });
    });
}

function render_brain(color: 'red' | 'black', brain: string) {
    let lines = brain.split('\n');
    let last_line = lines.pop();
    assert(last_line === '', last_line);

    let h = '<table class="brain">';
    lines.forEach((line, i) => {
        h += `<tr id="${color}-state-${i}">`;
        h += `<td class="state-no" data-state-no="${i}"></td>`;
        h += `<td>${line}</td>`;  // TODO: HTML entity escape
        h += '</tr>';
    })
    h += '</table>';
    document.getElementById(color + '-brain')!.innerHTML = h;
}

async function main() {
    let { hash } = window.location;
    assert(hash.startsWith('#'), hash);
    hash = hash.slice(1);
    let match = JSON.parse(decodeURIComponent(hash)) as Match;
    let r = await fetch('/background?match=' + encodeURIComponent(JSON.stringify(match)));
    assert(r.ok);
    let bg = await r.json() as Background;

    render_brain('red', bg.red_brain);
    render_brain('black', bg.black_brain);
    
    async function fetch_frame(frame_no: number) {
        r = await fetch('/frame?match=' + encodeURIComponent(JSON.stringify(match)) + '&frame_no=' + frame_no);
        assert(r.ok);
        return await r.json() as ReplayFrame;
    }

    let frame_no = 0;
    let frame = await fetch_frame(frame_no);
    document.getElementById('frame_no')!.innerText = '' + frame_no;

    async function change_frame(new_frame_no: number) {
        frame_no = new_frame_no;
        document.getElementById('frame_no')!.innerText = '' + frame.frame_no
            + (frame.frame_no == frame_no ? '' : '...');

        if (frame.frame_no == frame_no) {
            return;
        }
        
        let f = await fetch_frame(new_frame_no);
        if (Math.abs(f.frame_no - frame_no) < Math.abs(frame.frame_no - frame_no)) {
            frame = f;
            document.getElementById('frame_no')!.innerText = '' + frame.frame_no
                + (frame.frame_no == frame_no ? '' : '...');
            draw_stuff(tr);
            recompute_highlighted_state();
        }
    }

    let selected_ant_id: number | null = null;
    let highlighted_state: string | null = null;

    function update_highlighted_state(new_highlighted_state: string | null) {
        if (highlighted_state !== null) {
            document.getElementById(highlighted_state)!.classList.remove('highlighted');
        }
        highlighted_state = new_highlighted_state;
        if (highlighted_state !== null) {
            let el = document.getElementById(highlighted_state)!;
            el.classList.add('highlighted');
            el.scrollIntoView({ block: 'center' });
        }
    }

    function recompute_highlighted_state() {
        let new_highlighted_state = null;
        frame.ants.forEach((ant) => {
            if (ant.id === selected_ant_id) {
                new_highlighted_state = ant.color + '-state-' + ant.state;
            }
        });
        update_highlighted_state(new_highlighted_state);
    }

    let draw_stuff = (tr: Transform) => {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        draw_background(tr, bg);
        draw_frame(tr, frame);
        frame.ants.forEach((ant) => {
            if (ant.id === selected_ant_id) {
                let {x, y} = apply_transform(tr, ant.x, ant.y);
                ctx.strokeStyle = '#cc0';
                ctx.lineWidth = 4;
                ctx.beginPath();
                let r = tr.scale * 0.4;
                ctx.ellipse(x, y, r, r, 0, 0, 2 * Math.PI);
                ctx.stroke();
            }
        })
    };

    let max_x = Math.max(...bg.rocks.map(([x, y]) => x));
    let max_y = Math.max(...bg.rocks.map(([x, y]) => y));

    canvas = document.getElementById('canvas') as HTMLCanvasElement;
    ctx = canvas.getContext('2d')!;

    let hor_size = H_SCALE * (max_x + 1 + 0.5);
    let ver_size = (max_y + 1) * 0.75 + 0.25;
    let tr = {
        offset_x: 0,
        offset_y: 0,
        scale: Math.min(canvas.width / hor_size, canvas.height / ver_size),
    };
    draw_stuff(tr);

    document.onkeydown = e => {
        switch (e.code) {
            case 'ArrowRight':
                if (e.ctrlKey) {
                    change_frame(frame_no + 100);
                } else if (e.shiftKey) {
                    change_frame(frame_no + 10);
                } else {
                    change_frame(frame_no + 1);
                }
                break;
            case 'ArrowLeft':
                if (e.ctrlKey) {
                    change_frame(Math.max(frame_no - 100, 0));
                } else if (e.shiftKey) {
                    change_frame(Math.max(frame_no - 10, 0));
                } else {
                    change_frame(Math.max(frame_no - 1, 0));
                }
                break;
        }
    };

    canvas.onclick = (e) => {
        let r = canvas.getBoundingClientRect();
        let x = e.clientX - r.left;
        let y = e.clientY - r.top;
        let { row, col } = unapply_transform(tr, x, y);
        selected_ant_id = null;
        frame.ants.forEach((ant) => {
            if (ant.x == col && ant.y == row) {
                selected_ant_id = ant.id;
            }
        });
        draw_stuff(tr);
        recompute_highlighted_state();
    }

    canvas.onmousemove = (e) => {
        let r = canvas.getBoundingClientRect();
        let x = e.clientX - r.left;
        let y = e.clientY - r.top;
        let { row, col } = unapply_transform(tr, x, y);
        let tooltip = '(' + col + ', ' + row + ')';
        frame.food.forEach(([x, y, amount]) => {
            if (x == col && y == row) {
                tooltip += '\nfood: ' + amount;
            }
        })
        frame.ants.forEach((ant) => {
            if (ant.x == col && ant.y == row) {
                tooltip += '\n' +
                    ant.color + ' ant\n' +
                    '    id: ' + ant.id + '\n' +
                    '    state: ' + ant.state + '\n' +
                    '    resting: ' + ant.resting;
            }
        });
        canvas.title = tooltip;
    }

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
        zoom(tr, x, y, Math.exp(-delta * 0.002));
        requestAnimationFrame(() => draw_stuff(tr));
        e.preventDefault();
    };

    canvas.onpointermove = (e) => {
        if (e.buttons == 1) {
            tr.offset_x += e.movementX;
            tr.offset_y += e.movementY;
            requestAnimationFrame(() => draw_stuff(tr));
        }
    }
}

main()
