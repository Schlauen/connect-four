import { useEffect } from 'react';
import { AppState, WIDTH, useStore } from '../store';
import Column from './Column';
import { GameState, onUpdateGame } from '../Interface';
import { State } from './Cell';

const Playfield = () => {    
    const changeAppState = useStore(state => state.changeAppState);
    const setMessage = useStore(state => state.changeMessage);

    useEffect(() => {
        const unlisten = onUpdateGame(event => {
            console.log(event);

            if (event.state == GameState.Finished) {
                changeAppState(AppState.Finished);                
                if (event.winner != null) {
                    if (event.winner == State.Blank) {
                        setMessage('draw!');
                    }
                    else if (event.winner == State.P1) {
                        setMessage('P1 wins!');
                    }
                    else if (event.winner == State.P2) {
                        setMessage('P2 wins!');
                    }
                }
                
                setMessage('solved!')
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
