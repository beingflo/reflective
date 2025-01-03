import { createSignal, type Component, createEffect, For } from 'solid-js';

const View: Component = () => {
  const [images, setImages] = createSignal([]);

  createEffect(async () => {
    const response = await fetch('/api/images', {
      headers: {
        'content-type': 'application/json',
      },
    }).then((response) => response.json());

    setImages(response);
  });

  return (
    <div class="flex flex-col w-full px-4 md:px-0 md:w-1/3 mx-auto gap-8 md:gap-24 py-12">
      <For each={images()}>
        {(image) => (
          <div class="min--96">
            <img loading="lazy" src={`/api/images/${image}?quality=medium`} />
          </div>
        )}
      </For>
    </div>
  );
};

export default View;
