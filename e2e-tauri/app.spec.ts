describe("Sakya App (Real)", () => {
  it("should launch and display the welcome heading", async () => {
    const heading = await $("h1");
    await heading.waitForExist({ timeout: 10000 });
    const text = await heading.getText();
    expect(text).toMatch(/Welcome to Tauri/i);
  });

  it("should have the greet form", async () => {
    const input = await $("#greet-input");
    await input.waitForExist({ timeout: 5000 });
    expect(await input.isDisplayed()).toBe(true);

    const button = await $("button[type='submit']");
    expect(await button.isDisplayed()).toBe(true);
  });

  it("should greet when form is submitted", async () => {
    const input = await $("#greet-input");
    await input.setValue("E2E");

    const button = await $("button[type='submit']");
    await button.click();

    // Wait for the greeting to appear
    await browser.waitUntil(
      async () => {
        const body = await $("main").getText();
        return body.includes("Hello, E2E!");
      },
      { timeout: 10000, timeoutMsg: "Greeting did not appear" },
    );
  });
});
