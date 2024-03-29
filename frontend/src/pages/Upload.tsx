import { type Component, onMount, createEffect, createSignal } from 'solid-js';
import { useStore } from '../store';

const Upload: Component = () => {
  const [state] = useStore();
  const [images, setImages] = createSignal([]);
  let ref: HTMLInputElement;

  onMount(() => {
    ref.addEventListener('change', () => {
      setImages(Array.from(ref.files));
    });
  });

  const uploadImage = async (image: File) => {
    const formData = new FormData();
    formData.append('image', image, 'test');

    const response = await fetch('/api/images/upload', {
      body: formData,
      method: 'POST',
    }).then((response) => response.json());
  };

  createEffect(() => {
    images()?.forEach((image) => uploadImage(image));
  });

  return (
    <div class="mx-auto flex flex-col w-1/2 min-w-96 pt-12">
      <div class="flex flex-row gap-4 items-baseline">
        <p class="text-4xl md:text-6xl mb-4 text-black dark:text-white font-extrabold">
          Upload
        </p>
      </div>
      <label
        class="mt-8 text-center px-4 rounded-sm bg-white border border-black py-2 uppercase text-black hover:bg-gray-100 hover:shadow-none focus:outline-none hover:cursor-pointer"
        for="file"
      >
        Select files
      </label>
      <input ref={ref} type="file" id="file" class="hidden" multiple />
    </div>
  );
};

export default Upload;
