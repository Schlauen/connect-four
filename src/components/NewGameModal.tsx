import { useRef, useState } from 'react'
import "./Modal.css";
import { AppState, OpenModal, useStore } from '../store';
import { CellState, newGame } from '../Interface';
import Button from './Button';
import LevelRange from './LevelRange';
import LevelLabel from './LevelLabel';
import Checkbox from './Checkbox';

const NewGameModal = () => {
  const changeOpenModal = useStore(state => state.changeOpenModal);
  const changeAppState = useStore(state => state.changeAppState);

  const onError = useStore(state => state.changeMessage);
  const level = useStore(state => state.level);
  const [computerStarts, setComputerStarts] = useState(false);

  return (
    <div className='modal-background'>
        <div className='modal-container'>
            <div className='title'>
                <h1>New Game</h1>
            </div>
            <LevelRange min={2} max={10}/>
            <LevelLabel/>
            <Checkbox 
                name='Computer starts'
                onStateToggle={setComputerStarts}
            />
            <Button
                name='start'
                onClick={() => {
                  newGame(
                    level,
                    computerStarts ? CellState.P2 : CellState.P1,
                    onError, 
                    () => {
                      changeAppState(AppState.Playing) 
                    }
                  );
                  changeOpenModal(OpenModal.None);
                }}
            />
        </div>
    </div>
  )
}

export default NewGameModal
