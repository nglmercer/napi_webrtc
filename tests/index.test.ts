import { expect, test, describe } from "bun:test";
import { WebRTCAPI } from "../index";

describe("WebRTC NAPI Module Tests", () => {
  test("Should create WebRTCAPI instance", () => {
    const api = new WebRTCAPI();
    expect(api).toBeDefined();
  });

  test("Should create a PeerConnection", async () => {
    const api = new WebRTCAPI();
    const pc = await api.createPeerConnection();
    expect(pc).toBeDefined();
    await pc.close();
  });

  test("Should create an offer", async () => {
    const api = new WebRTCAPI();
    const pc = await api.createPeerConnection();
    try {
      const offer = await pc.createOffer();
      expect(typeof offer).toBe("string");
      expect(offer.length).toBeGreaterThan(0);
    } finally {
      await pc.close();
    }
  });
});
