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
import { createVisibilityObserver } from '@solid-primitives/intersection-observer';

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

  const openLightbox = (imageId: string) => {
    setOpenImage(imageId);
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

  let [numImages, setNumImages] = createSignal(10);

  let [leftImages, setLeftImages] = createSignal([]);
  let [middleImages, setMiddleImages] = createSignal([]);
  let [rightImages, setRightImages] = createSignal([]);

  createEffect(() => {
    let heightLeft = 0;
    let heightMiddle = 0;
    let heightRight = 0;

    let imagesLeft = [];
    let imagesMiddle = [];
    let imagesRight = [];

    const images = state.images.slice(0, numImages());

    images.forEach((image) => {
      const imageHeight = 1 / image.aspect_ratio;
      if (heightLeft <= heightMiddle && heightLeft <= heightRight) {
        heightLeft += imageHeight;
        imagesLeft.push(image);
      } else if (heightMiddle <= heightLeft && heightMiddle <= heightRight) {
        heightMiddle += imageHeight;
        imagesMiddle.push(image);
      } else if (heightRight <= heightLeft && heightRight <= heightMiddle) {
        heightRight += imageHeight;
        imagesRight.push(image);
      }
    });
    setLeftImages(imagesLeft);
    setMiddleImages(imagesMiddle);
    setRightImages(imagesRight);
  });

  let el: HTMLDivElement | undefined;

  const visible = createVisibilityObserver({
    threshold: 1.0,
    rootMargin: '800px',
  })(() => el);

  createEffect(() => {
    if (visible()) {
      setNumImages((prev) => prev + 20);
    }
  });

  return (
    <div>
      <Show when={state.images.length === 0}>
        <div class="flex w-full h-96">
          <div class="m-auto flex flex-col gap-4">
            <h1 class="text-4xl text-center ">No images found</h1>
            <p class="text-center">
              Press <span class="font-bold">U</span> to upload
            </p>
          </div>
        </div>
      </Show>
      <Show when={openImage()}>
        <Lightbox imageId={openImage()} />
      </Show>
      <div class="flex flex-col w-full">
        <div class="flex flex-row gap-4 p-8 max-w-screen-2xl mx-auto">
          <div class="flex flex-col gap-4 w-1/3">
            <For each={leftImages()}>
              {(image) => (
                <div class={`w-full aspect-[${image.aspect_ratio}] h-auto`}>
                  <img
                    class="object-fill w-full"
                    id={image?.id}
                    onClick={() => openLightbox(image?.id)}
                    src={`/api/images/${image?.id}?quality=small`}
                  />
                </div>
              )}
            </For>
          </div>
          <div class="flex flex-col gap-4 w-1/3">
            <For each={middleImages()}>
              {(image) => (
                <div class={`w-full aspect-[${image.aspect_ratio}] h-auto`}>
                  <img
                    class={`object-fill w-full aspect-[${image.aspect_ratio}]`}
                    id={image?.id}
                    onClick={() => openLightbox(image?.id)}
                    src={`/api/images/${image?.id}?quality=small`}
                  />
                </div>
              )}
            </For>
          </div>
          <div class="flex flex-col gap-4 w-1/3">
            <For each={rightImages()}>
              {(image) => (
                <div class={`w-full aspect-[${image.aspect_ratio}] h-auto`}>
                  <img
                    class={`object-fill w-full aspect-[${image.aspect_ratio}]`}
                    id={image?.id}
                    onClick={() => openLightbox(image?.id)}
                    src={`/api/images/${image?.id}?quality=small`}
                  />
                </div>
              )}
            </For>
          </div>
        </div>
        <div ref={el} class="h-1" />
      </div>
    </div>
  );
};

export default View;
