// Types used to communicate with the server.
// Keep in sync with vis_server.rs.

export interface Match {
    world: string,
    red: string,
    black: string,
}

export interface Background {
    rocks: [number, number][],
    red_anthill: [number, number][],
    black_anthill: [number, number][],
}

export interface ReplayFrame {
    frame_no: number,
    food: [number, number, number][],
    ants: Ant[],
}

export interface Ant {
    color: 'red' | 'black',
    x: number,
    y: number,
    dir: number,
    has_food: boolean,
}