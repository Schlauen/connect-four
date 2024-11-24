import { event, invoke } from "@tauri-apps/api";
import { UnlistenFn, listen } from "@tauri-apps/api/event";

export interface Update {
    Cell: CellUpdate,
    State: StateUpdate,
    Balance: BalanceUpdate,
}

export interface CellUpdate {
    row: number,
    col: number,
    state: number,
    winning: boolean,
}

export interface StateUpdate {
    state: number,
    winner: number,
}

export interface BalanceUpdate {
    value: number,
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
    Calculating: 3,
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
    level:number,
    startingPlayer:number,
    onError: (msg:string) => void,
    onSuccess: () => void, 
) {
    invoke(
        'new_game',
        {
            level:level,
            startingPlayer:startingPlayer
        }
    ).then(onSuccess)
    .catch(onError);
}


export function onUpdateCell(row:number, col:number, onTrigger: (event:Update) => void): Promise<UnlistenFn> {
    console.log('update cell', event);
    return listen<Update>('updateCell-' + row + '-' + col, event => onTrigger(event.payload));
}

export function onUpdateState(onTrigger: (event:Update) => void): Promise<UnlistenFn> {
    console.log('update state', event);
    return listen<Update>('updateState', event => onTrigger(event.payload));
}

export function onUpdateBalance(onTrigger: (event:Update) => void): Promise<UnlistenFn> {
    console.log('update balance', event);
    return listen<Update>('updateBalance', event => onTrigger(event.payload));
}