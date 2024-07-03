import Footer from './Footer';
import Header from './Header';
import { AppState, useStore } from '../store';
import StartSidebar from './StartSidebar';
import Playfield from './Playfield';

const renderSidebar = (appState:number) => {
  {
    switch (appState) {
      case AppState.Start:
      case AppState.Playing:
      case AppState.Finished:
        return <StartSidebar/>
    }   
  }
}

const MainFrame = () => {
  const appState = useStore(state => state.appState);

  return (
    <div id='main-frame'>
      <Header/>
      {
        renderSidebar(appState)
      }
      <Playfield/>
      <Footer />
    </div>
  )
}

export default MainFrame