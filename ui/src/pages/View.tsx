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

  // Load to top of page on refresh
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
