import {
  type Component,
  onMount,
  createEffect,
  createSignal,
  For,
  Show,
} from 'solid-js';
import { limitFunction } from 'p-limit';

const Upload: Component = () => {
  const [images, setImages] = createSignal([]);
  const [imageStates, setImageStates] = createSignal({});
  const [uploadComplete, setUploadComplete] = createSignal(false);
  let ref: HTMLInputElement;

  const initializeImageStates = (files: File[]) => {
    return files.reduce((acc, file) => {
      acc[file.name] = 'waiting';
      return acc;
    }, {});
  };

  onMount(() => {
    ref.addEventListener('change', () => {
      const files = Array.from(ref.files);
      setImages(files);
      setImageStates(initializeImageStates(files));
    });
  });

  const handleDrop = (event: DragEvent) => {
    event.preventDefault();
    const files = Array.from(event.dataTransfer.files);
    setImages(files);
    setImageStates(initializeImageStates(files));
  };

  const handleDragOver = (event: DragEvent) => {
    event.preventDefault();
  };

  const uploadImage = limitFunction(
    async (image: File) => {
      setImageStates((prev) => ({ ...prev, [image.name]: 'uploading' }));
      const formData = new FormData();
      formData.append('image', image, 'test');

      await fetch('/api/images/upload', {
        body: formData,
        method: 'POST',
      });

      setImageStates((prev) => ({ ...prev, [image.name]: 'done' }));
    },
    { concurrency: 6 },
  );

  createEffect(() => {
    if (
      Object.entries(imageStates())?.length > 0 &&
      Object.values(imageStates()).every((state) => state === 'done')
    ) {
      setImages([]);
      setUploadComplete(true);
    }
  });

  createEffect(() => {
    images()?.forEach((image: any) => uploadImage(image));
  });

  return (
    <div class="mx-auto flex flex-col w-1/2 min-w-96 py-12">
      <div class="flex flex-row gap-4 items-baseline">
        <p class="text-4xl md:text-6xl mb-4 text-black dark:text-white font-extrabold">
          Upload
        </p>
      </div>
      <Show when={uploadComplete()}>
        <div class="text-emerald-700 mt-4">Upload complete!</div>
      </Show>
      <Show when={images()?.length === 0}>
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
      </Show>
      <ul class="mt-4">
        <For each={images()}>
          {(image) => (
            <Show when={imageStates()[image.name] !== 'done'}>
              <li class="flex justify-between">
                <span>{image.name}</span>
                <span>{imageStates()[image.name]}</span>
              </li>
            </Show>
          )}
        </For>
      </ul>
    </div>
  );
};

export default Upload;
