import { useNavigate, useParams } from '@solidjs/router';
import { createEffect, onCleanup, type Component } from 'solid-js';
import { useStore } from '../store';
import { tinykeys } from 'tinykeys';

export const validateEvent = (callback) => (event) => {
  if (event.target.tagName !== 'INPUT' && event.target.tagName !== 'TEXTAREA') {
    event.preventDefault();
    callback();
  }
};

const Lightbox: Component = () => {
  const params = useParams();
  const navigate = useNavigate();
  const [state, { setImages }] = useStore();

  // Fetch images in case of reload
  createEffect(async () => {
    if (state.images.length === 0) {
      const response = await fetch('/api/images', {
        headers: {
          'content-type': 'application/json',
        },
      }).then((response) => response.json());

      setImages(response);
    }
  });

  const goToNextImage = () => {
    const currentIndex = state.images.indexOf(params.id);
    const nextImage = state.images[currentIndex + 1];

    if (nextImage) {
      navigate(`/${nextImage}`);
    }
  };

  const goToLastImage = () => {
    const currentIndex = state.images.indexOf(params.id);
    const lastImage = state.images[currentIndex - 1];

    if (lastImage) {
      navigate(`/${lastImage}`);
    }
  };

  const cleanup = tinykeys(window, {
    ArrowRight: validateEvent(goToNextImage),
    ArrowLeft: validateEvent(goToLastImage),
    Escape: () => navigate('/'),
  });

  onCleanup(cleanup);

  return (
    <div class="flex w-full h-screen p-2 md:p-8 justify-center">
      <img
        class="h-full w-full object-contain"
        src={`/api/images/${params.id}?quality=original`}
      />
    </div>
  );
};

export default Lightbox;
