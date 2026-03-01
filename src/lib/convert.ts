export async function convertAvatar(file?: File): Promise<Blob | null> {
    if (!file) return null;

    return new Promise((resolve) => {
        const canvas = document.createElement("canvas");
        const context = canvas.getContext("2d", { alpha: true });
        const image = new Image();

        image.addEventListener("load", () => {
            canvas.width = image.width;
            canvas.height = image.height;
            context?.drawImage(image, 0, 0);

            canvas.toBlob((blob) => resolve(blob), "image/webp", 0.9);
        });
        image.src = URL.createObjectURL(file);
    });
}
