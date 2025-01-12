import { JSX, createContext, createEffect, useContext } from 'solid-js';
import { createStore } from 'solid-js/store';

export const storeName = 'store';

const StoreContext = createContext<any[]>();

export type State = {
  images: Array<String>;
};

export const [state, setState] = createStore({ images: [] });

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
    },
  ];

  return (
    <StoreContext.Provider value={store}>
      {props.children}
    </StoreContext.Provider>
  );
}

export const useStore = () => useContext(StoreContext);
