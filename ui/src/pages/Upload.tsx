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

  const handleDrop = (event: DragEvent) => {
    event.preventDefault();
    setImages(Array.from(event.dataTransfer.files));
  };

  const handleDragOver = (event: DragEvent) => {
    event.preventDefault();
  };

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
      <div
        class="h-60 mt-8 flex flex-col items-center justify-center border border-dashed border-black py-12"
        onDrop={handleDrop}
        onDragOver={handleDragOver}
      >
        <label for="file" class="cursor-pointer">
          <p class="text-black underline inline">Select files</p>
          <p class="text-black inline"> or drag and drop files here</p>
        </label>
        <input ref={ref} type="file" id="file" class="hidden" multiple />
      </div>
    </div>
  );
};

export default Upload;
