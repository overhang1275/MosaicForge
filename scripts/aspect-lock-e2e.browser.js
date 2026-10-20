async function aspectLockE2e(page) {
  const ensure = (condition, message) => {
    if (!condition) throw new Error(message);
  };
  const errors = [],
    candidates = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("response", (response) => {
    if (new URL(response.url()).pathname === "/api/preview")
      candidates.push(
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
  await page.locator("summary").filter({ hasText: "Transformaciones" }).click();
  const lock = page.getByRole("checkbox", {
    name: "Mantener proporciones (X = Y)",
    exact: true,
  });
  const x = page.getByRole("group", { name: "Escala X", exact: true }),
    y = page.getByRole("group", { name: "Escala Y", exact: true });
  ensure(await lock.isChecked(), "default must preserve proportions");
  await x.getByLabel("Mínimo", { exact: true }).fill("1.1");
  await x.getByLabel("Máximo", { exact: true }).fill("1.8");
  ensure(
    (await y.getByLabel("Mínimo", { exact: true }).inputValue()) === "1.1",
    "X minimum must update Y",
  );
  await y.getByLabel("Máximo", { exact: true }).fill("2.1");
  ensure(
    (await x.getByLabel("Máximo", { exact: true }).inputValue()) === "2.1",
    "Y maximum must update X",
  );
  await lock.focus();
  await lock.press("Space");
  ensure(!(await lock.isChecked()), "keyboard must unlock");
  await y.getByLabel("Máximo", { exact: true }).fill("2.5");
  ensure(
    (await x.getByLabel("Máximo", { exact: true }).inputValue()) === "2.1",
    "unlocked axes must be independent",
  );
  await lock.focus();
  await lock.press("Space");
  ensure(await lock.isChecked(), "keyboard must relock");
  ensure(
    (await y.getByLabel("Máximo", { exact: true }).inputValue()) === "2.1",
    "relocking copies X range",
  );
  await page.getByLabel("Variantes de muestra", { exact: true }).fill("3");
  const firstRequest = page.waitForRequest(
    (request) => new URL(request.url()).pathname === "/api/preview",
  );
  await page
    .getByRole("button", { name: "Generar variantes", exact: true })
    .click();
  const firstGeneration = JSON.parse(
    (await firstRequest).postData(),
  ).generation;
  ensure(
    firstGeneration.rulesVersion === "2" &&
      firstGeneration.rules.preserveAspectRatio === true,
    "request must carry the versioned lock",
  );
  ensure(
    JSON.stringify(firstGeneration.rules.scaleX) ===
      JSON.stringify(firstGeneration.rules.scaleY),
    "locked ranges must match",
  );
  await page
    .getByText("Serie terminada. Selecciona una variante para revisarla.", {
      exact: true,
    })
    .waitFor({ timeout: 120000 });
  const rows = await Promise.all(candidates);
  const accepted = rows.filter(
    (row) => row.status === 200 && row.variant.status === "accepted",
  );
  ensure(
    accepted.length === 3 &&
      accepted.every((row) => row.variant.rulesVersion === "2"),
    "engine must return three accepted rules2 variants",
  );
  for (const image of await page.locator(".gallery-grid img").all()) {
    await image.scrollIntoViewIfNeeded();
    await image.evaluate((element) => element.decode());
  }
  await lock.uncheck();
  await x.getByLabel("Máximo", { exact: true }).fill("2.4");
  const pdfRequest = page.waitForRequest(
    (request) =>
      new URL(request.url()).pathname === "/api/export-pdf-download",
  );
  const pdfResponse = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname === "/api/export-pdf-download",
    { timeout: 120000 },
  );
  const download = page.waitForEvent("download", { timeout: 120000 });
  await page
    .getByRole("button", { name: "Descargar PDF multipágina", exact: true })
    .click();
  const saved = JSON.parse((await pdfRequest).postData());
  ensure(
    saved.job.generation.rules.preserveAspectRatio === true,
    "PDF must retain generated lock after control edits",
  );
  ensure((await pdfResponse).status() === 200, "vector PDF must export");
  const file = await download;
  ensure((await file.failure()) === null, "PDF download must complete");
  await file.saveAs("out/aspect/locked.pdf");
  await lock.check();
  await lock.scrollIntoViewIfNeeded();
  await page.screenshot({ path: "out/aspect/desktop.png", fullPage: false });
  await page.setViewportSize({ width: 390, height: 844 });
  await lock.scrollIntoViewIfNeeded();
  ensure(
    await page.evaluate(
      () => document.documentElement.scrollWidth === innerWidth,
    ),
    "mobile must not overflow",
  );
  await page.screenshot({ path: "out/aspect/mobile.png", fullPage: false });
  ensure(errors.length === 0, "no uncaught browser exceptions");
  return {
    checks: [
      "default lock and bidirectional range edits",
      "keyboard unlock/relock with independent ranges",
      "three real accepted rules2 previews",
      "saved lock preserved in complete PDF after edits",
      "desktop/mobile without horizontal overflow",
    ],
    firstGeneration,
    pdfJob: saved.job,
    indices: saved.variantIndices,
    errors,
  };
}
