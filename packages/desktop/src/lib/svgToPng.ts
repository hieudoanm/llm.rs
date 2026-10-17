export const svgToPngBlob = async (svgElement: SVGSVGElement, pixelRatio = 2): Promise<Blob> => {
  const svgMarkup = new XMLSerializer().serializeToString(svgElement);
  const svgDataUrl = `data:image/svg+xml;base64,${btoa(svgMarkup)}`;
  const img = await loadImage(svgDataUrl);
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d');
  if (!ctx) {
    throw new Error('Failed to get canvas 2d context');
  }
  canvas.width = img.width * pixelRatio;
  canvas.height = img.height * pixelRatio;
  ctx.scale(pixelRatio, pixelRatio);
  ctx.drawImage(img, 0, 0);
  return new Promise<Blob>((resolve, reject) => {
    canvas.toBlob(
      (blob) => (blob ? resolve(blob) : reject(new Error('PNG conversion failed'))),
      'image/png',
    );
  });
};

const loadImage = (src: string): Promise<HTMLImageElement> =>
  new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = src;
  });
