import { createSignal, type Component, onMount, createEffect } from 'solid-js';
import { useStore } from '../store';

const Upload: Component = () => {
  const [state, { setImages }] = useStore();
  const [files, setFiles] = createSignal();
  const [uploadLinks, setUploadLinks] = createSignal([]);
  let ref: HTMLInputElement;

  createEffect(() => console.log(uploadLinks()));

  onMount(() => {
    ref.addEventListener('change', () => {
      setImages(ref.files);

      fetch((uploadLinks() as Array<{ original: string }>)[0].original, {
        body: ref.files[0],
        method: 'PUT',
      }).catch(() => null);
    });
  });

  fetch('/api/images/upload', {
    body: JSON.stringify({ number: 1 }),
    method: 'POST',
    headers: {
      'content-type': 'application/json',
    },
  })
    .then((response) => response.json())
    .then((body) => setUploadLinks(body))
    .catch(() => null);

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
