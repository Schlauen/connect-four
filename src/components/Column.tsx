import Cell from "./Cell";
import { HEIGHT, useStore } from "../store";
import { playCol } from "../Interface";

interface Props {
    col: number;
}

const Column = ({col: col}: Props) => {
    const onError = useStore(state => state.changeMessage);
    return (
        <div 
            className='col'
            id={'col-' + col}
            key={'col-' + col}
            onClick={() => playCol(col, onError)}
        >
            {Array(HEIGHT).fill(undefined).map((_, row) => <Cell row={HEIGHT-1-row} col={col}/>)}
        </div>
    )
}

export default Column
