import { WebRTCAPI } from "../index";

async function main() {
  // close by timeut 2 minutes
  setTimeout(() => {
    // close process after 2 minutes
    console.log("Closing process after 2 minutes...");
    process.exit(0);
  }, 120000);

  console.log("Starting WebRTC NAPI example...");
  
  const api = new WebRTCAPI();
  console.log("WebRTC API initialized");

  // Create two peer connections to simulate local communication
  const pc1 = await api.createPeerConnection();
  const pc2 = await api.createPeerConnection();
  console.log("PeerConnections created");

  // Set up ICE candidate handlers
  pc1.onIceCandidate((candidate) => {
    if (candidate) {
      console.log("PC1 found ICE candidate");
      pc2.addIceCandidate(candidate).catch(e => console.error("PC2 addIceCandidate error:", e));
    }
  });

  pc2.onIceCandidate((candidate) => {
    if (candidate) {
      console.log("PC2 found ICE candidate");
      pc1.addIceCandidate(candidate).catch(e => console.error("PC1 addIceCandidate error:", e));
    }
  });

  // Set up DataChannel on PC1
  const dc1 = await pc1.createDataChannel("chat");
  console.log("DataChannel 'chat' created on PC1");

  dc1.onOpen((err) => {
    if (err) {
      console.error("PC1 DataChannel open error:", err);
      return;
    }
    console.log("DataChannel on PC1 opened!");
    dc1.send("Hello from PC1!").catch(console.error);
  });

  dc1.onMessage((data) => {
    console.log("PC1 received message:", data);
  });

  // Set up DataChannel handler on PC2
  pc2.onDataChannel((dc2) => {
    console.log("PC2 received DataChannel:", dc2.label());
    dc2.onOpen((err) => {
      if (err) {
        console.error("PC2 DataChannel open error:", err);
        return;
      }
      console.log("DataChannel on PC2 opened!");
      dc2.send("Hello from PC2!").catch(console.error);
    });
    dc2.onMessage((data) => {
      console.log("PC2 received message:", data);
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

  // Wait a bit for ICE candidates and data channel to open
  console.log("Waiting for connection...");
  await new Promise(resolve => setTimeout(resolve, 5000));

  console.log("Closing connections...");
  await pc1.close();
  await pc2.close();
  console.log("Done");
}

main().catch(console.error);
