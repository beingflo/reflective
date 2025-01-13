import { useNavigate } from '@solidjs/router';
import {
  type Component,
  createEffect,
  createSignal,
  For,
  onCleanup,
  Show,
} from 'solid-js';
import { useStore } from '../store';
import Lightbox from '../components/Lightbox';
import { tinykeys } from 'tinykeys';

export const validateEvent = (callback) => (event) => {
  if (event.target.tagName !== 'INPUT' && event.target.tagName !== 'TEXTAREA') {
    event.preventDefault();
    callback();
  }
};

const View: Component = () => {
  const [state, { setImages }] = useStore();
  const [openImage, setOpenImage] = createSignal('');

  const goToNextImage = () => {
    if (!openImage()) return;
    const currentIndex = state.images.indexOf(openImage());
    const nextImage = state.images[currentIndex + 1];

    if (nextImage) {
      setOpenImage(nextImage);
    }
  };

  const goToLastImage = () => {
    if (!openImage()) return;
    const currentIndex = state.images.indexOf(openImage());
    const lastImage = state.images[currentIndex - 1];

    if (lastImage) {
      setOpenImage(lastImage);
    }
  };

  const closeLightbox = () => {
    document
      .getElementById(openImage())
      ?.scrollIntoView({ behavior: 'smooth' });
    setOpenImage('');
  };

  const cleanup = tinykeys(window, {
    ArrowRight: validateEvent(goToNextImage),
    ArrowLeft: validateEvent(goToLastImage),
    Escape: closeLightbox,
  });

  onCleanup(cleanup);

  createEffect(async () => {
    const response = await fetch('/api/images', {
      headers: {
        'content-type': 'application/json',
      },
    }).then((response) => response.json());

    setImages(response);
  });

  const openLightbox = (image: string) => {
    setOpenImage(image);
  };

  return (
    <div>
      <Show when={openImage()}>
        <Lightbox imageId={openImage()} />
      </Show>
      <div class="grid grid-cols-1 md:grid-cols-3 md:w-3/4 px-4 py-4 mx-auto gap-8">
        <For each={state.images}>
          {(image) => (
            <div class="aspect-square w-full">
              <img
                class="object-cover w-full h-full"
                id={image}
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
