import { create } from 'zustand';

export const AppState = {
    Start: 0,
    Playing: 1,
    Finished: 2,
}

export const OpenModal = {
    None: 0,
    NewGame: 1,
}

export const WIDTH = 7;
export const HEIGHT = 6;

type GameState = {
    message: string;
    changeMessage: (newMessage:string) => void;
    controlsEnabled: boolean;
    setControlsEnabled: (enabled: boolean) => void;
    appState: number;
    changeAppState: (newAppState:number) => void;
    openModal: number;
    changeOpenModal: (newOpenModal:number) => void;
}

export const useStore = create<GameState>((set) => ({
    message: 'Welcome to Connect Four!',
    changeMessage: newMessage => set({message: newMessage}),
    controlsEnabled: true,
    setControlsEnabled: enabled => set({controlsEnabled: enabled}),
    appState: AppState.Start,
    changeAppState: newAppState => set({appState: newAppState}),
    openModal: OpenModal.None,
    changeOpenModal: newOpenModal => set(state => {
        if (newOpenModal != OpenModal.None) {
            state.controlsEnabled = false;
        } else {
            state.controlsEnabled = true;
        }
        return {openModal: newOpenModal}
    }),
}));