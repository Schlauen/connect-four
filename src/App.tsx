import "./App.css";
import MainFrame from "./components/MainFrame";
import NewGameModal from "./components/NewGameModal";
import { OpenModal, useStore } from "./store";

const renderModal = (openModal: number) => {
  {
    switch (openModal) {
      case OpenModal.NewGame:
        return <NewGameModal/>
    }
  }
}

function App() {
  const openModal = useStore(state => state.openModal);
  return (
    <div className="container">
      <MainFrame/>
      {
        renderModal(openModal)
      }
    </div>
  );
}

export default App;
