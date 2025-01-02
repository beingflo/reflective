export const getImageUploadLinks = async (number: number) => {
  const response = await fetch('/images/upload', {
    body: JSON.stringify({ number }),
    method: 'POST',
  });
  // TODO handle response
};

export const getImageList = async () => {
  const response = await fetch('/images');
  // TODO handle response
};
