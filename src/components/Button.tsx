interface Props {
    name: string;
    onClick: () => void;
}

const Button = ({name, onClick} : Props) => {
    return (
        <div 
            className='menu-element btn'
            onClick={onClick}
        >
            {name}
        </div>
    )
}

export default Button
