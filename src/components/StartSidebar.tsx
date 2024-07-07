import { OpenModal, useStore } from '../store';
import BalanceOfPower from './BalanceOfPower';
import Button from './Button';
import LevelLabel from './LevelLabel';

const StartSidebar = () => {
    const changeOpenModal = useStore(state => state.changeOpenModal);

    return (
        <div id='sidebar'>
            <Button
                name='new game'
                onClick={() => changeOpenModal(OpenModal.NewGame)}
            />
            <LevelLabel/>
            <BalanceOfPower/>
        </div>
    )
}

export default StartSidebar
