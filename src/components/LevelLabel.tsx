import { useStore } from "../store";

const LevelLabel = () => {
  const level:number = useStore(state => state.level);
  return (
    <div className='menu-element key-value'>
        <label>level:</label>
        <label>{level}</label>
    </div>
  )
}

export default LevelLabel
