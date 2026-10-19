async function acceptedCountE2e(page) {
  const ensure = (condition, message) => {
    if (!condition) throw new Error(message);
  };
  const root = page.url();
  ensure(/^http:\/\/127\.0\.0\.1:\d+\/$/.test(root), "Open local editor first");
  const errors = [],
    pending = [],
    checks = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("response", (response) => {
    if (new URL(response.url()).pathname === "/api/preview")
      pending.push(
        response.json().then((body) => ({
          status: response.status(),
          variant: body.variant,
        })),
      );
  });
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.reload();
  await page
    .getByLabel("Archivo SVG", { exact: true })
    .setInputFiles("docs/fixtures/editor-seed.svg");
  await page
    .getByRole("button", { name: "Importar plantilla", exact: true })
    .click();
  await page
    .getByText("Plantilla importada. Ajusta las reglas y genera variantes.", {
      exact: true,
    })
    .waitFor();
  await page.getByLabel("Variantes de muestra", { exact: true }).fill("12");
  await page.locator("summary").filter({ hasText: "Transformaciones" }).click();
  const range = async (name, min, max) => {
    const group = page.getByRole("group", { name, exact: true });
    await group.getByLabel("Mínimo", { exact: true }).fill(min);
    await group.getByLabel("Máximo", { exact: true }).fill(max);
  };
  await range("Escala X", "1", "1.3");
  await range("Escala Y", "1", "1.3");
  await page
    .getByRole("button", { name: "Generar variantes", exact: true })
    .click();
  await page
    .getByText("Serie terminada. Selecciona una variante para revisarla.", {
      exact: true,
    })
    .waitFor({ timeout: 120000 });
  const responses = await Promise.all(pending);
  ensure(
    responses.every((response) => response.status === 200),
    "all candidate requests must complete normally",
  );
  const accepted = responses
    .filter((response) => response.variant.status === "accepted")
    .map((response) => response.variant.index);
  const rejected = responses
    .filter((response) => response.variant.status === "rejected")
    .map((response) => response.variant.index);
  ensure(
    accepted.length === 12 && rejected.length > 0,
    "require twelve accepted after actual geometric rejections",
  );
  ensure(new Set(accepted).size === 12, "accepted indices must be distinct");
  ensure(
    (await page.locator(".gallery-grid img").count()) === 12,
    "gallery must have twelve images",
  );
  await page.waitForFunction(
    () =>
      [...document.querySelectorAll(".gallery-grid img")].length === 12 &&
      [...document.querySelectorAll(".gallery-grid img")].every(
        (image) => image.complete && image.naturalWidth > 0,
      ),
  );
  checks.push(
    "real rejected candidates replaced until twelve decodable accepted previews",
  );
  await page.screenshot({
    path: "out/accepted/twelve-desktop.png",
    fullPage: true,
  });
  const pdfButton = page.getByRole("button", {
    name: "Descargar PDF multipágina",
    exact: true,
  });
  const requestPromise = page.waitForRequest(
    (request) =>
      new URL(request.url()).pathname === "/api/export-pdf-selection",
  );
  const responsePromise = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname === "/api/export-pdf-selection",
  );
  const downloadPromise = page.waitForEvent("download");
  await pdfButton.click();
  const selection = JSON.parse((await requestPromise).postData());
  const exported = await responsePromise;
  ensure(exported.status() === 200, "selected PDF must export successfully");
  ensure(
    JSON.stringify(selection.variantIndices) === JSON.stringify(accepted),
    "PDF must use every accepted index in order",
  );
  const download = await downloadPromise;
  ensure((await download.failure()) === null, "complete PDF download required");
  await download.saveAs("out/accepted/twelve.pdf");
  checks.push(
    "one vector PDF uses the complete nonconsecutive accepted selection",
  );
  await page.getByLabel("Semilla", { exact: true }).fill("999");
  const replayRequest = page.waitForRequest(
    (request) =>
      new URL(request.url()).pathname === "/api/export-pdf-selection",
  );
  const replayResponse = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname === "/api/export-pdf-selection",
  );
  const replayDownload = page.waitForEvent("download");
  await pdfButton.click();
  ensure(
    JSON.stringify(
      JSON.parse((await replayRequest).postData()).variantIndices,
    ) === JSON.stringify(accepted),
    "editing controls must not alter selection",
  );
  const replay = await replayResponse;
  ensure(replay.status() === 200, "replay export must succeed");
  const savedReplay = await replayDownload;
  ensure(
    (await savedReplay.failure()) === null,
    "replay download must complete",
  );
  await savedReplay.saveAs("out/accepted/twelve-replay.pdf");
  checks.push(
    "saved selection remains immutable after control edits; replay saved for byte verification",
  );
  await page.setViewportSize({ width: 390, height: 844 });
  for (const image of await page.locator(".gallery-grid img").all()) {
    await image.scrollIntoViewIfNeeded();
    await image.evaluate((element) => element.decode());
  }
  ensure(
    await page.evaluate(
      () => document.documentElement.scrollWidth === innerWidth,
    ),
    "mobile must not overflow horizontally",
  );
  await page.screenshot({
    path: "out/accepted/twelve-mobile.png",
    fullPage: true,
  });
  checks.push("mobile width 390 without horizontal overflow");
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.getByLabel("Variantes de muestra", { exact: true }).fill("1");
  await range("Traslación X", "0", "0");
  await range("Traslación Y", "0", "0");
  await range("Rotación (°)", "45", "45");
  await range("Escala X", "1", "1");
  await range("Escala Y", "1", "1");
  const before = pending.length;
  await page
    .getByRole("button", { name: "Generar variantes", exact: true })
    .click();
  await page
    .getByText("Serie incompleta: 0 de 1 variantes aceptadas.", { exact: true })
    .waitFor({ timeout: 60000 });
  const failed = await Promise.all(pending.slice(before));
  ensure(
    failed.length === 10 &&
      failed.every(
        (response) =>
          response.status === 200 && response.variant.status === "rejected",
      ),
    "impossible rules must stop at ten real rejected attempts",
  );
  ensure(await pdfButton.isDisabled(), "incomplete target must block PDF");
  await page
    .getByRole("alert")
    .filter({ hasText: "tras 10 intentos" })
    .waitFor();
  checks.push(
    "impossible rules fail explicitly at bounded attempts with no partial PDF",
  );
  ensure(errors.length === 0, "no uncaught browser exceptions");
  return {
    checks,
    acceptedIndices: accepted,
    rejectedIndices: rejected,
    attempts: responses.length,
    selection,
    errors,
  };
}
