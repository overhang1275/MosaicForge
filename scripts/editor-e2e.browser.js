async function editorE2e(page) {
  // Run from repository root with Playwright CLI run-code --filename=... .
  const base = page.url();
  if (!/^http:\/\/127\.0\.0\.1:\d+\/$/.test(base))
    throw new Error("Open the local editor root before running this check.");
  const ensure = (value, message) => {
    if (!value) throw new Error(message);
  };
  const checks = [],
    errors = [],
    requests = [],
    downloads = [];
  page.on("download", (download) => downloads.push(download));
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("request", (request) => {
    if (new URL(request.url()).pathname.startsWith("/api/"))
      requests.push({
        path: new URL(request.url()).pathname,
        body: request.postData(),
      });
  });
  const generate = () =>
    page
      .getByRole("button", { name: "Generar variantes", exact: true })
      .click();
  const finished = () =>
    page
      .getByText("Serie terminada. Selecciona una variante para revisarla.", {
        exact: true,
      })
      .waitFor({ timeout: 60000 });
  const pdfButton = page.getByRole("button", {
    name: "Descargar PDF multipágina",
    exact: true,
  });
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.reload();
  // Invalid source travels to the real importer; no synthetic engine response.
  await page.getByLabel("Archivo SVG", { exact: true }).setInputFiles({
    name: "invalid.svg",
    mimeType: "image/svg+xml",
    buffer: Buffer.from("<svg><broken>"),
  });
  const invalidImport = page.waitForResponse(
    (response) => new URL(response.url()).pathname === "/api/inspect",
  );
  await page
    .getByRole("button", { name: "Importar plantilla", exact: true })
    .click();
  const invalidResponse = await invalidImport;
  ensure(
    invalidResponse.status() === 400,
    "invalid SVG must receive validation error, not server failure",
  );
  ensure(
    (await invalidResponse.json()).code === "invalid_template",
    "real importer must reject the SVG",
  );
  await page
    .getByRole("alert")
    .filter({ hasText: "SVG no admitido o inválido" })
    .waitFor();
  ensure(await pdfButton.isDisabled(), "invalid SVG must not enable PDF");
  checks.push("real invalid SVG rejected");
  await page
    .getByLabel("Archivo SVG", { exact: true })
    .setInputFiles("docs/fixtures/editor-seed.svg");
  const beforeImport = requests.length;
  ensure(beforeImport === 1, "choosing a file must not send it");
  await page
    .getByRole("button", { name: "Importar plantilla", exact: true })
    .click();
  await page
    .getByText("Plantilla importada. Ajusta las reglas y genera variantes.", {
      exact: true,
    })
    .waitFor();
  await page.getByLabel("Variantes de muestra", { exact: true }).fill("3");
  await generate();
  await finished();
  ensure(
    (await page.locator(".gallery-grid img").count()) === 3,
    "three accepted real previews required",
  );
  const first = JSON.parse(
    requests.find((r) => r.path === "/api/preview").body,
  );
  await page.getByLabel("Semilla", { exact: true }).fill("2027");
  await generate();
  await finished();
  const previewRequests = requests.filter((r) => r.path === "/api/preview");
  ensure(
    previewRequests.length === 6,
    "each generation must issue three sequential previews",
  );
  ensure(
    JSON.parse(previewRequests[3].body).generation.seed === "2027",
    "changed rule must reach the engine",
  );
  checks.push("real import, three previews, seed change, regenerate");
  await page.getByLabel("Semilla", { exact: true }).fill("2028");
  const pdfRequest = page.waitForRequest(
    (r) => new URL(r.url()).pathname === "/api/export-pdf-selection",
  );
  const downloadPromise = page.waitForEvent("download");
  await pdfButton.click();
  const pdf = JSON.parse((await pdfRequest).postData());
  const download = await downloadPromise;
  ensure((await download.failure()) === null, "PDF download must finish");
  await download.saveAs("out/phase26/editor.pdf");
  ensure(
    downloads.length === 1,
    "the series must produce exactly one download",
  );
  ensure(
    pdf.job.generation.seed === "2027" && pdf.job.generation.count === 3,
    "PDF must use all saved cards, not edited controls",
  );
  checks.push("one multipage PDF uses immutable saved generation");
  // Generation limits must fail locally before any engine request.
  await page.getByLabel("Variantes de muestra", { exact: true }).fill("13");
  const beforeLimit = requests.length;
  await generate();
  await page.getByRole("alert").filter({ hasText: "entre 1 y 12" }).waitFor();
  ensure(requests.length === beforeLimit, "limit must fail before transport");
  checks.push("count limit fails before network");
  await page.getByLabel("Variantes de muestra", { exact: true }).fill("1");
  await page.locator("summary").filter({ hasText: "Transformaciones" }).click();
  for (const [name, value] of [
    ["Rotación (°)", "45"],
    ["Escala X", "1"],
    ["Escala Y", "1"],
  ]) {
    const group = page.getByRole("group", { name, exact: true });
    await group.getByLabel("Mínimo", { exact: true }).fill(value);
    await group.getByLabel("Máximo", { exact: true }).fill(value);
  }
  const rejection = page.waitForResponse(
    (response) => new URL(response.url()).pathname === "/api/preview",
  );
  await generate();
  await page
    .getByText("Serie incompleta: 0 de 1 variantes aceptadas.", { exact: true })
    .waitFor({ timeout: 60000 });
  const rejectedResponse = await rejection;
  ensure(
    rejectedResponse.status() === 200 &&
      (await rejectedResponse.json()).variant.status === "rejected",
    "require actual geometric rejection rather than transport error",
  );
  ensure(
    (await page.locator(".gallery-grid img").count()) === 0,
    "rejected variant must not invent image",
  );
  ensure(
    await pdfButton.isDisabled(),
    "rejected series must disable complete PDF",
  );
  checks.push("real geometric rejection blocks PDF");
  for (const [name, value] of [
    ["Rotación (°)", "0"],
    ["Escala X", "2"],
    ["Escala Y", "2"],
  ]) {
    const group = page.getByRole("group", { name, exact: true });
    await group.getByLabel("Mínimo", { exact: true }).fill(value);
    await group.getByLabel("Máximo", { exact: true }).fill(value);
  }
  // Keep transport real, delaying only delivery so stale results can be exercised.
  let release;
  const held = new Promise((resolve) => {
    release = resolve;
  });
  let captured;
  const entered = new Promise((resolve) => {
    captured = resolve;
  });
  await page.route("**/api/preview", async (route) => {
    const response = await route.fetch();
    captured((await response.json()).variant.status);
    await held;
    await route.fulfill({ response }).catch(() => {});
  });
  await generate();
  ensure(
    (await entered) === "accepted",
    "the delayed real response must be accepted",
  );
  const abortedRequest = page.waitForEvent(
    "requestfailed",
    (request) => new URL(request.url()).pathname === "/api/preview",
  );
  await page
    .getByRole("button", { name: "Cancelar generación", exact: true })
    .click();
  await abortedRequest;
  await page
    .getByText(
      "Generación cancelada. El motor puede seguir terminando la operación actual.",
      { exact: true },
    )
    .waitFor();
  release();
  await page.unrouteAll({ behavior: "wait" });
  await page.evaluate(
    () =>
      new Promise((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(resolve)),
      ),
  );
  ensure(await pdfButton.isDisabled(), "cancelled series must block PDF");
  ensure(
    (await page.locator(".gallery-grid img").count()) === 0,
    "stale cancelled response must not publish an image",
  );
  checks.push(
    "controlled response delay: cancellation aborts browser request and blocks partial PDF",
  );
  ensure(errors.length === 0, "no uncaught browser exceptions");
  await page.screenshot({
    path: "out/phase26/editor-after.png",
    fullPage: true,
  });
  return { checks, errors, firstGeneration: first.generation, pdfJob: pdf.job };
}
