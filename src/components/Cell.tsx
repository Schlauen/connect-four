import { useEffect, useState } from "react";
import { onUpdateCell } from "../Interface";

export const State = {
    Blank: 0,
    P1: 1,
    P2: -1,
    Winning: 2
}

interface Props {
    row: number;
    col: number;
}

interface Cell {
    state: number,
}

function getClassName(state:number, winning:boolean) {
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
        className = '';
    }

    if (winning) {
        className += ' win';
    }

    return className;
}

const Cell = ({ row, col }: Props) => {
    const [state, setState] = useState(State.Blank);
    const [winning, setWinning] = useState(false);

    useEffect(() => {
        const unlisten = onUpdateCell(row, col, event => {
            if (event.Cell.state != null) {
                setState(event.Cell.state);
            }
            if (event.Cell.winning != null) {
                setWinning(event.Cell.winning);
            }
        });
    
        return () => {
            unlisten.then(f => f());
        };
    });
    
    return (
        <div 
            id={row  + "," + col }
            key={row + "," + col }
            className={getClassName(state, winning)}
        />
    );
};

export default Cell;
