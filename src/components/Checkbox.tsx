import { useState } from 'react'

interface Props {
    name: string;
    onStateToggle: (state:boolean) => void; 
}

const Checkbox = ({name, onStateToggle} : Props) => {
    const [active, setActive] = useState(false);

    return (
        <div className='menu-element checkbox-container'>
            <label>{name}</label>
            <div 
                className='custom-checkbox'
                onClick={() => {
                    const newState = !active;
                    setActive(newState)
                    onStateToggle(newState);
                }}
            >
                {active ? <div className='tick'></div> : undefined}
            </div>
        </div>
    )
};

export default Checkbox
