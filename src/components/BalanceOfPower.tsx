import { useEffect, useState } from "react";
import { onUpdateBalance } from "../Interface";

const BalanceOfPower = () => {
  const [bop, setBop] = useState(0);

  useEffect(() => {
    const unlisten = onUpdateBalance(event => {
        if (event.Balance.value != null) {
          setBop(event.Balance.value);
        }
    });

    return () => {
        unlisten.then(f => f());
    };
  });
  
  return (
    <div className='menu-element bop' style={{gridTemplateColumns: (bop + 127) / 254 * 100 + "% auto"}}>
        <div id="bop-p1"/>
        <div id='bop-p2'/>
    </div>
  )
}

export default BalanceOfPower
