import { Match } from './types.js';

document.getElementById('run-button')!.onclick = (e) => {
    let world = (document.getElementById('world') as HTMLInputElement).value;
    let red = (document.getElementById('red-brain') as HTMLInputElement).value;
    let black = (document.getElementById('black-brain') as HTMLInputElement).value;
    let seed = parseInt((document.getElementById('seed') as HTMLInputElement).value);
    let match: Match = {
        world,
        red,
        black,
        seed,
    };
    window.location.href = '/vis/index.html#' + encodeURIComponent(JSON.stringify(match));
};
