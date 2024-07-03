import { useRef } from 'react'
import "./Modal.css";
import { AppState, OpenModal, useStore } from '../store';
import { newGame } from '../Interface';
import Button from './Button';
import Range from './Range';

const NewGameModal = () => {
  const rangeRef = useRef<any>(null);
  const changeOpenModal = useStore(state => state.changeOpenModal);
  const changeAppState = useStore(state => state.changeAppState);

  const onError = useStore(state => state.changeMessage);

  return (
    <div className='modal-background'>
        <div className='modal-container'>
            <div className='title'>
                <h1>configuration</h1>
            </div>
            <Range min={2} max={10} ref={rangeRef}/>
            <Button
                name='start'
                onClick={() => {
                  newGame(onError, () => {
                    changeAppState(AppState.Playing) 
                  });
                  changeOpenModal(OpenModal.None);
                }}
            />
        </div>
    </div>
  )
}

export default NewGameModal
