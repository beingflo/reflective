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
import { validateEvent } from '../utils';

const View: Component = () => {
  const [state, { setImages }] = useStore();
  const [openImage, setOpenImage] = createSignal('');
  const navigate = useNavigate();

  const goToNextImage = () => {
    if (!openImage()) return;
    const currentIndex = state.images.findIndex(
      (image) => image.id === openImage(),
    );
    const nextImage = state.images[currentIndex + 1];

    if (nextImage) {
      setOpenImage(nextImage?.id);
    }
  };

  const goToLastImage = () => {
    if (!openImage()) return;
    const currentIndex = state.images.findIndex(
      (image) => image.id === openImage(),
    );
    const lastImage = state.images[currentIndex - 1];

    if (lastImage) {
      setOpenImage(lastImage?.id);
    }
  };

  const closeLightbox = () => {
    document
      .getElementById(openImage())
      ?.scrollIntoView({ behavior: 'instant', block: 'center' });
    setOpenImage('');
  };

  const cleanup = tinykeys(window, {
    ArrowRight: validateEvent(goToNextImage),
    ArrowLeft: validateEvent(goToLastImage),
    Escape: closeLightbox,
    u: validateEvent(() => navigate('/upload')),
  });

  onCleanup(cleanup);

  // Scroll to top of page on refresh
  window.onbeforeunload = function () {
    window.scrollTo(0, 0);
  };

  createEffect(async () => {
    const response = await fetch('/api/images', {
      headers: {
        'content-type': 'application/json',
      },
    }).catch((error) => {
      console.error('Failed to fetch images:', error);
      throw error;
    });

    if (response.status === 401) {
      navigate('/login');
      return;
    }

    const data = await response.json();
    setImages(data);
  });

  const openLightbox = (imageId: string) => {
    setOpenImage(imageId);
  };

  const leftImages = () => state.images.filter((_, idx) => idx % 3 === 0);
  const middleImages = () => state.images.filter((_, idx) => idx % 3 === 1);
  const rightImages = () => state.images.filter((_, idx) => idx % 3 === 2);

  return (
    <div>
      <Show when={openImage()}>
        <Lightbox imageId={openImage()} />
      </Show>
      <div class="flex flex-row gap-4 p-8 max-w-screen-2xl mx-auto">
        <div class="flex flex-col gap-4 w-1/3">
          <For each={leftImages()}>
            {(image) => (
              <img
                class="object-fill w-full min-h-24"
                id={image?.id}
                loading="lazy"
                onClick={() => openLightbox(image?.id)}
                src={`/api/images/${image?.id}?quality=small`}
              />
            )}
          </For>
        </div>
        <div class="flex flex-col gap-4 w-1/3">
          <For each={middleImages()}>
            {(image) => (
              <img
                class="object-fill w-full min-h-24"
                id={image?.id}
                loading="lazy"
                onClick={() => openLightbox(image?.id)}
                src={`/api/images/${image?.id}?quality=small`}
              />
            )}
          </For>
        </div>
        <div class="flex flex-col gap-4 w-1/3">
          <For each={rightImages()}>
            {(image) => (
              <img
                class="object-fill w-full min-h-24"
                id={image?.id}
                loading="lazy"
                onClick={() => openLightbox(image?.id)}
                src={`/api/images/${image?.id}?quality=small`}
              />
            )}
          </For>
        </div>
      </div>
    </div>
  );
};

export default View;
