import pkg from "../index.js";
const { WebRTCAPI } = pkg;

async function main() {
  let pc1: any, pc2: any;
  try {
    console.log("Starting WebRTC NAPI example...");
    
    const api = new WebRTCAPI();
    console.log("WebRTC API initialized");

    // Create two peer connections to simulate local communication
    pc1 = await api.createPeerConnection();
    pc2 = await api.createPeerConnection();
    console.log("PeerConnections created");

    // Set up ICE candidate handlers
    pc1.onIceCandidate((err: any, candidateJson: string | null) => {
      if (err) {
        console.error("PC1 onIceCandidate error:", err);
        return;
      }
      if (candidateJson) {
        const candidate = JSON.parse(candidateJson);
        console.log("PC1 found ICE candidate:", candidate.candidate);
        pc2.addIceCandidate(candidate).catch((e: any) => console.error("PC2 addIceCandidate error:", e));
      } else {
        console.log("PC1 ICE gathering finished");
      }
    });

    pc2.onIceCandidate((err: any, candidateJson: string | null) => {
      if (err) {
        console.error("PC2 onIceCandidate error:", err);
        return;
      }
      if (candidateJson) {
        const candidate = JSON.parse(candidateJson);
        console.log("PC2 found ICE candidate:", candidate.candidate);
        pc1.addIceCandidate(candidate).catch((e: any) => console.error("PC1 addIceCandidate error:", e));
      } else {
        console.log("PC2 ICE gathering finished");
      }
    });

  // Set up DataChannel on PC1
    const dc1 = await pc1.createDataChannel("chat");
    console.log("DataChannel 'chat' created on PC1");

    dc1.onOpen((err: any) => {
      if (err) {
        console.error("PC1 DataChannel open error:", err);
        return;
      }
      console.log("DataChannel on PC1 opened!");
      dc1.send("Hello from PC1!").catch(console.error);
    });

    // Wait for messages from both sides
    console.log("Waiting for connection and messages...");
    
    const pc1Received = new Promise<void>(resolve => {
      dc1.onMessage((err: any, data: string) => {
        if (err) {
          console.error("PC1 onMessage error:", err);
          return;
        }
        console.log("PC1 received message:", data);
        resolve();
      });
    });

    let dc2Resolve: () => void;
    const pc2Received = new Promise<void>(resolve => {
      dc2Resolve = resolve;
    });

    // Set up DataChannel handler on PC2
    pc2.onDataChannel((err: any, dc2: any) => {
      if (err) {
        console.error("PC2 onDataChannel error:", err);
        return;
      }
      console.log("PC2 received DataChannel:", dc2.label());
      dc2.onOpen((err: any) => {
        if (err) {
          console.error("PC2 DataChannel open error:", err);
          return;
        }
        console.log("DataChannel on PC2 opened!");
        dc2.send("Hello from PC2!").catch(console.error);
      });
      dc2.onMessage((err: any, data: string) => {
        if (err) {
          console.error("PC2 onMessage error:", err);
          return;
        }
        console.log("PC2 received message:", data);
        dc2Resolve();
      });
    });

  // Negotiation flow
  console.log("Starting negotiation...");
  
  // 1. PC1 creates offer
  const offer = await pc1.createOffer();
  console.log("PC1 offer created");

  // 2. PC2 sets remote description (offer)
  await pc2.setRemoteDescription(offer, "offer");
  console.log("PC2 remote description set");

  // 3. PC2 creates answer
  const answer = await pc2.createAnswer();
  console.log("PC2 answer created");

  // 4. PC1 sets remote description (answer)
  await pc1.setRemoteDescription(answer, "answer");
  console.log("PC1 remote description set");

  // Wait for both messages to be received with a timeout
  await Promise.race([
    Promise.all([pc1Received, pc2Received]),
    new Promise((_, reject) => setTimeout(() => reject(new Error("Timeout waiting for messages")), 10000))
  ]);

  } catch (error) {
    console.error("Error in main:", error);
    process.exit(1);
  } finally {
    console.log("Closing connections...");
    if (pc1) await pc1.close().catch(() => {});
    if (pc2) await pc2.close().catch(() => {});
    console.log("Done");
    process.exit(0);
  }
}

main();
