import { forwardRef, useImperativeHandle, useState } from 'react'

interface Props {
    min:number;
    max:number;
}

const Range = forwardRef(({min, max} : Props, ref) => {
    const [value, setValue] = useState(max);

    const getValue = () => value;
    useImperativeHandle(ref, () => ({getValue}));
    return (
        <div className='menu-element range-container'>
            <label>level:</label>
            <input type='range' min={min} max={max} value={value} className='slider' onChange={(val) => {
                setValue(Number(val.target.value));
            }}/>
        </div>
    )
});

export default Range
