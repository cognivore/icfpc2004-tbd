// Types used to communicate with the server.
// Keep in sync with vis_server.rs.

export interface Match {
    world: string,
    red: string,
    black: string,
    seed: number,
}

export interface Background {
    rocks: [number, number][],
    red_anthill: [number, number][],
    black_anthill: [number, number][],
    red_brain: string,
    black_brain: string,
}

export interface ReplayFrame {
    frame_no: number,
    food: [number, number, number][],
    ants: Ant[],
    red_markers: [number, number, boolean[]][],
    black_markers: [number, number, boolean[]][],
}

export interface Ant {
    id: number,
    color: 'red' | 'black',
    x: number,
    y: number,
    dir: number,
    has_food: boolean,
    state: number,
    resting: number,
}
