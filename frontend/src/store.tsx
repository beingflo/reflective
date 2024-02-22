import { JSX, createContext, createEffect, useContext } from 'solid-js';
import { createStore } from 'solid-js/store';

export const storeName = 'store';

const StoreContext = createContext({});

const localState = localStorage.getItem(storeName);

export type State = {
  images: Array<string>;
};

const parsedState: State = localState ? (JSON.parse(localState) as State) : { images: [] };

export const [state, setState] = createStore(parsedState);

export type StoreProviderProps = {
  children: JSX.Element;
};

export function StoreProvider(props: StoreProviderProps) {
  createEffect(() => localStorage.setItem(storeName, JSON.stringify(state)));

  const store = [
    state,
    {
      setImages(images: Array<string>) {
        setState({ images });
      },
    },
  ];

  return <StoreContext.Provider value={store}>{props.children}</StoreContext.Provider>;
}

export const useStore = () => useContext(StoreContext);
