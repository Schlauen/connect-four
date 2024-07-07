import { useEffect, useState } from "react";
import { onUpdateGame } from "../Interface";

const BalanceOfPower = () => {
  const [bop, setBop] = useState(0);

  useEffect(() => {
    const unlisten = onUpdateGame(event => {
        if (event.balance_of_power != null) {
          console.log(event.balance_of_power);
          setBop(event.balance_of_power);
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
