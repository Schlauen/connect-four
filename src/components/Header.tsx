import { useEffect } from 'react'
import { AppState, useStore } from '../store';

const Header = () => {
    const setMessage = useStore(state => state.changeMessage);
    const appState = useStore(state => state.appState);
    
    useEffect(() => {
        const interval = setInterval(
            function() {
                if (appState == AppState.Finished) return;
                setMessage('');
            }, 
            3000
        );

        return () => clearInterval(interval);
    })

    const message = useStore(state => state.message);

    return (
        <div id='header'>
            {message}
        </div>
    )
};

export default Header
