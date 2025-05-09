import {
  type Component,
  onMount,
  createEffect,
  createSignal,
  For,
  Show,
  onCleanup,
} from 'solid-js';
import { limitFunction } from 'p-limit';
import { useNavigate } from '@solidjs/router';
import { tinykeys } from 'tinykeys';

const Upload: Component = () => {
  const [images, setImages] = createSignal([]);
  const [imageStates, setImageStates] = createSignal({});
  const [_, setUploadComplete] = createSignal(false);
  let ref: HTMLInputElement;
  const navigate = useNavigate();

  let goHome = () => {
    if (images().length === 0) {
      navigate('/');
    }
  };

  const cleanup = tinykeys(window, {
    Escape: goHome,
  });

  onCleanup(cleanup);

  const uploaded = () =>
    Object.values(imageStates()).filter((state) => state === 'done').length;

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
      formData.append('filename', image.name);
      formData.append('data', image);
      formData.append('last_modified', image.lastModified.toString());

      const response = await fetch('/api/images', {
        body: formData,
        method: 'POST',
      }).catch((error) => {
        console.error('Failed to upload image:', error);
        throw error;
      });

      if (response.status === 401) {
        navigate('/login');
      } else {
        setImageStates((prev) => ({ ...prev, [image.name]: 'done' }));
      }
    },
    { concurrency: 16 },
  );

  createEffect(() => {
    if (
      Object.entries(imageStates())?.length > 0 &&
      Object.values(imageStates()).every((state) => state === 'done')
    ) {
      setImages([]);
      setUploadComplete(true);
      setImageStates({});
    }
  });

  createEffect(() => {
    images()?.forEach((image: any) => uploadImage(image));
  });

  return (
    <div class="mx-auto flex flex-col w-full px-12 md:w-2/3 min-w-96 py-12">
      <div class="flex flex-row gap-4 items-baseline">
        <p class="text-4xl md:text-6xl mb-4 text-black dark:text-white font-extrabold">
          Upload
        </p>
      </div>
      <Show when={images().length === 0}>
        <div
          class="h-60 mt-8 flex flex-col items-center justify-center border border-dashed border-black py-12"
          onDrop={handleDrop}
          onDragOver={handleDragOver}
        >
          <label for="file" class="cursor-pointer">
            <p class="text-black underline inline">Select files</p>
            <p class="text-black inline"> or drag and drop files here</p>
          </label>
          <input
            disabled={images()?.length > 0}
            ref={ref}
            type="file"
            id="file"
            class="hidden"
            multiple
          />
        </div>
      </Show>
      <Show when={Object.values(imageStates())?.length > 0}>
        <div class="mt-4">
          Uploading {uploaded()} / {Object.values(imageStates())?.length}
        </div>
      </Show>
      <div class="mt-4 grid grid-cols-1 md:grid-cols-3 xl:grid-cols-5 gap-4">
        <For each={images()}>
          {(image) => (
            <Show when={imageStates()[image.name] !== 'done'}>
              <div class="flex flex-col gap-2">
                <div class="border border-black border-dashed rounded-sm w-24 h-16" />
                <div>{image.name}</div>
              </div>
            </Show>
          )}
        </For>
      </div>
    </div>
  );
};

export default Upload;
