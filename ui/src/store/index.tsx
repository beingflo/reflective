import { JSX, createContext, createEffect, useContext } from 'solid-js';
import { createStore } from 'solid-js/store';
import { Screen } from '../types';

export const storeName = 'store';

const StoreContext = createContext<any[]>();

export type State = {
  images: Array<String>;
  screen: Screen;
};

export const [state, setState] = createStore({ images: [], screen: 'app' });

export type StoreProviderProps = {
  children: JSX.Element;
};

export function StoreProvider(props: StoreProviderProps) {
  const store = [
    state,
    {
      setImages(images: Array<String>) {
        setState({ images });
      },
      cycleScreen(screen: Screen) {
        const currentScreen = state.screen;
        let newScreen: Screen = 'app';
        if (currentScreen !== screen) {
          newScreen = screen;
        }
        setState({ screen: newScreen });
      },
    },
  ];

  return (
    <StoreContext.Provider value={store}>
      {props.children}
    </StoreContext.Provider>
  );
}

export const useStore = () => useContext(StoreContext);
