/** Assert explicit workflow transitions and value history in the complete source-owned SFCs. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

async function expectOutput(page: Page, text: string): Promise<void> {
  await page.locator("output").filter({ hasText: text }).waitFor();
  assert.equal((await page.locator("output").textContent())?.trim(), text);
}

async function activate(page: Page, label: string): Promise<void> {
  await page.getByRole("button", { name: label, exact: true }).focus();
  await page.keyboard.press("Enter");
}

async function buttons(page: Page): Promise<string[]> {
  return (await page.getByRole("button").allTextContents()).map((text) => text.trim());
}

export async function exerciseWorkflowStateExample(page: Page, name: string): Promise<string[]> {
  if (name === "use-machine") {
    await expectOutput(page, "Step: cart · Items: 0");
    assert.deepEqual(await buttons(page), ["Add item"]);
    await page.getByText("Add an item before checking out.").waitFor();
    await activate(page, "Add item");
    await expectOutput(page, "Step: cart · Items: 1");
    assert.deepEqual(await buttons(page), ["Add item", "Go to shipping"]);
    await activate(page, "Go to shipping");
    await expectOutput(page, "Step: shipping · Items: 1");
    await activate(page, "Back");
    await expectOutput(page, "Step: cart · Items: 1");
    await activate(page, "Go to shipping");
    await activate(page, "Review order");
    await activate(page, "Place order");
    await expectOutput(page, "Step: placed · Items: 1");
    assert.deepEqual(await buttons(page), ["Start a new order"]);
    await activate(page, "Start a new order");
    await expectOutput(page, "Step: cart · Items: 0");
    return [
      "guard hides checkout for an empty cart",
      "only valid steps are offered",
      "restart resets context",
    ];
  }
  if (name === "use-previous") {
    const price = page.getByRole("spinbutton", { name: "Ticket price" });
    await expectOutput(page, "No earlier price yet.");
    await price.fill("150");
    await expectOutput(page, "Up 30 from 120.");
    await price.fill("90");
    await expectOutput(page, "Down 60 from 150.");
    return ["first change reports the replaced value", "later changes compare with the last value"];
  }
  throw new Error(`No workflow state law for ${name}`);
}
