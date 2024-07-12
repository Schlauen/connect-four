import { useEffect } from 'react';
import { AppState, WIDTH, useStore } from '../store';
import Column from './Column';
import { GameState, onUpdateState } from '../Interface';
import { State } from './Cell';

const Playfield = () => {    
    const changeAppState = useStore(state => state.changeAppState);
    const setMessage = useStore(state => state.changeMessage);

    useEffect(() => {
        const unlisten = onUpdateState(event => {
            if (event.State.state == GameState.Finished) {
                changeAppState(AppState.Finished);                
                if (event.State.winner != null) {
                    if (event.State.winner == State.Blank) {
                        setMessage('draw!');
                    }
                    else if (event.State.winner == State.P1) {
                        setMessage('Player 1 wins!');
                    }
                    else if (event.State.winner == State.P2) {
                        setMessage('Player 2 wins!');
                    }
                }
            }
            else if (event.State.state == GameState.Calculating) {             
                setMessage('thinking...');
            }
            else {
                setMessage('');
            }
        });
    
        return () => {unlisten.then(f => f())};
    });

    return (
        <div id='playfield'>
            {Array(WIDTH).fill(undefined).map((_, col) => <Column col={col}/>)}
        </div>
    )
};

export default Playfield
