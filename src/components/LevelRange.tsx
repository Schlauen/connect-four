import { forwardRef, useImperativeHandle, useState } from 'react'
import { useStore } from '../store';

interface Props {
    min:number;
    max:number;
}

const LevelRange = ({min, max} : Props) => {
    const selLevel = useStore(state => state.setLevel);
    const level = useStore(state => state.level);
    return (
        <div className='menu-element range-container'>
            <input type='range' min={min} max={max} value={level} className='slider' onChange={(val) => {
                selLevel(Number(val.target.value));
            }}/>
        </div>
    )
};

export default LevelRange
