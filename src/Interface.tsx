import { invoke } from "@tauri-apps/api";
import { UnlistenFn, listen } from "@tauri-apps/api/event";

export interface CellUpdateEvent {
    row: number,
    col: number,
    state: number,
}

export interface GameUpdateEvent {
    state: number,
    winner: number,
    cell_updates: [CellUpdateEvent],
}

export const CellState = {
    Blank: 0,
    P1: 1,
    P2: -1,
}

export const GameState = {
    Blank: 0,
    Running: 1,
    Finished: 2,
}

export function playCol(
    col:number,
    onError: (msg:string) => void
) {
    invoke(
        'play_col', 
        {
            col:col,
        }
    ).then(_ => {})
    .catch(onError);
}

export function newGame(
    onError: (msg:string) => void,
    onSuccess: () => void, 
) {
    invoke(
        'new_game',
    ).then(onSuccess)
    .catch(onError);
}


export function onUpdateCell(row:number, col:number, onTrigger: (event:CellUpdateEvent) => void): Promise<UnlistenFn> {
    return listen<CellUpdateEvent>('updateCell-' + row + '-' + col, event => onTrigger(event.payload));
}

export function onUpdateGame(onTrigger: (event:GameUpdateEvent) => void): Promise<UnlistenFn> {
    return listen<GameUpdateEvent>('updateGame', event => onTrigger(event.payload));
}