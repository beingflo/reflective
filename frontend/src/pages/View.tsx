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
    <div class="flex flex-col w-1/2 mx-auto gap-24 py-12">
      <For each={images()}>
        {(image) => (
          <div class="min-h-96">
            <img loading="lazy" src={`/api/images/${image}?quality=original`} />
          </div>
        )}
      </For>
    </div>
  );
};

export default View;
