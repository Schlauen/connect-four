import { useEffect, useState } from "react";
import { onUpdateCell } from "../Interface";

export const State = {
    Blank: 0,
    P1: 1,
    P2: -1,
}

interface Props {
    row: number;
    col: number;
}

interface Cell {
    state: number,
}

function getClassName(state:number) {
    let className:string;
    if (state == State.Blank) {
        className = 'cell blank';
    }
    else if (state == State.P1) {
        className = 'cell p1';
    }
    else if (state == State.P2) {
        className = 'cell p2';
    }
    else {
        console.log(state);
        className = '';
    }

    return className;
}

const Cell = ({ row, col }: Props) => {
    const [state, setState] = useState(State.Blank);

    useEffect(() => {
        const unlisten = onUpdateCell(row, col, event => {
            if (event.state != null) {
                setState(event.state);
            }
            console.log(event);
        });
    
        return () => {
            unlisten.then(f => f());
        };
    });
    
    return (
        <div 
            id={row  + "," + col }
            key={row + "," + col }
            className={getClassName(state)}
        />
    );
};

export default Cell;
