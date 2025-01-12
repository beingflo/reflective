import { useNavigate } from '@solidjs/router';
import { type Component, createEffect, For } from 'solid-js';
import { useStore } from '../store';

const View: Component = () => {
  const [state, { setImages }] = useStore();
  const navigate = useNavigate();

  createEffect(async () => {
    const response = await fetch('/api/images', {
      headers: {
        'content-type': 'application/json',
      },
    }).then((response) => response.json());

    setImages(response);
  });

  const openLightbox = (image: string) => {
    navigate(`/${image}`);
  };

  return (
    <div>
      <div class="grid grid-cols-1 md:grid-cols-3 md:w-3/4 px-4 py-4 mx-auto gap-8">
        <For each={state.images}>
          {(image) => (
            <div class="aspect-square w-full">
              <img
                class="object-cover w-full h-full"
                loading="lazy"
                onClick={() => openLightbox(image)}
                src={`/api/images/${image}?quality=small`}
              />
            </div>
          )}
        </For>
      </div>
    </div>
  );
};

export default View;
