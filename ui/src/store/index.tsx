import { JSX, createContext, createEffect, useContext } from 'solid-js';
import { createStore } from 'solid-js/store';
import { Screen } from '../types';

export const storeName = 'store';

const StoreContext = createContext<any[]>();

export type Image = {
  id: string;
  captured_at: string;
  aspect_ratio: number;
  tags: Array<string>;
};

export type State = {
  images: Array<Image>;
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
      appendImages(images: Array<String>) {
        setState((state) => ({
          images: [...(state.images ?? []), ...(images ?? [])],
        }));
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
