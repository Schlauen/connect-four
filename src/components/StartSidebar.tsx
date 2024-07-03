import { AppState, OpenModal, useStore } from '../store';
import Button from './Button';

const StartSidebar = () => {
    const changeOpenModal = useStore(state => state.changeOpenModal);

    return (
        <div id='sidebar'>
            <Button
                name='new game'
                onClick={() => () => changeOpenModal(OpenModal.NewGame)}
            />
        </div>
    )
}

export default StartSidebar
