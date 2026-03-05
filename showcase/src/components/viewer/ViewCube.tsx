"use client";

import React, { useEffect, useRef } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { ViewPresetName } from "@/lib/viewer-types";

export interface ViewCubeProps {
  cameraRef: React.MutableRefObject<THREE.Camera | null>;
  controlsRef: React.MutableRefObject<OrbitControls | null>;
  onApplyViewPreset: (preset: ViewPresetName) => void;
}

export function ViewCube({ cameraRef, controlsRef, onApplyViewPreset }: ViewCubeProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const width = 80;
    const height = 80;

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(50, width / height, 0.1, 100);
    camera.position.z = 4;

    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    renderer.setSize(width, height);
    renderer.setPixelRatio(window.devicePixelRatio);
    containerRef.current.appendChild(renderer.domElement);

    const materials = [
      new THREE.MeshBasicMaterial({ color: 0x334455, wireframe: false, transparent: true, opacity: 0.9 }), // Right
      new THREE.MeshBasicMaterial({ color: 0x334455, wireframe: false, transparent: true, opacity: 0.9 }), // Left
      new THREE.MeshBasicMaterial({ color: 0x445566, wireframe: false, transparent: true, opacity: 0.9 }), // Top
      new THREE.MeshBasicMaterial({ color: 0x445566, wireframe: false, transparent: true, opacity: 0.9 }), // Bottom
      new THREE.MeshBasicMaterial({ color: 0x556677, wireframe: false, transparent: true, opacity: 0.9 }), // Front
      new THREE.MeshBasicMaterial({ color: 0x556677, wireframe: false, transparent: true, opacity: 0.9 }), // Back
    ];

    const geometry = new THREE.BoxGeometry(1.6, 1.6, 1.6);
    const edges = new THREE.EdgesGeometry(geometry);
    const edgeMaterial = new THREE.LineBasicMaterial({ color: 0x8899aa, linewidth: 2 });
    
    // Create an edge helper
    const edgeLines = new THREE.LineSegments(edges, edgeMaterial);

    const cube = new THREE.Mesh(geometry, materials);
    cube.add(edgeLines);
    scene.add(cube);

    const ambientLight = new THREE.AmbientLight(0xffffff, 0.8);
    scene.add(ambientLight);

    const dirLight = new THREE.DirectionalLight(0xffffff, 0.5);
    dirLight.position.set(2, 4, 3);
    scene.add(dirLight);

    let animationFrameId: number;

    const animate = () => {
      animationFrameId = requestAnimationFrame(animate);

      if (cameraRef.current) {
        // Sync rotation with main camera
        cube.quaternion.copy(cameraRef.current.quaternion).invert();
      }

      renderer.render(scene, camera);
    };

    animate();

    // Interaction handling
    const raycaster = new THREE.Raycaster();
    const mouse = new THREE.Vector2();

    const onClick = (event: MouseEvent) => {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      mouse.x = ((event.clientX - rect.left) / width) * 2 - 1;
      mouse.y = -((event.clientY - rect.top) / height) * 2 + 1;

      raycaster.setFromCamera(mouse, camera);
      const intersects = raycaster.intersectObject(cube);

      if (intersects.length > 0) {
        const faceIndex = intersects[0].face?.materialIndex;
        switch (faceIndex) {
          case 0: onApplyViewPreset("right"); break;
          case 1: onApplyViewPreset("left"); break;
          case 2: onApplyViewPreset("top"); break;
          case 3: onApplyViewPreset("bottom"); break;
          case 4: onApplyViewPreset("front"); break;
          case 5: onApplyViewPreset("back"); break;
        }
      }
    };

    const canvas = renderer.domElement;
    canvas.addEventListener("click", onClick);
    canvas.style.cursor = "pointer";

    return () => {
      cancelAnimationFrame(animationFrameId);
      canvas.removeEventListener("click", onClick);
      containerRef.current?.removeChild(renderer.domElement);
      renderer.dispose();
      geometry.dispose();
      materials.forEach(m => m.dispose());
      edges.dispose();
      edgeMaterial.dispose();
    };
  }, [cameraRef, controlsRef, onApplyViewPreset]);

  return (
    <div
      ref={containerRef}
      style={{
        position: "absolute",
        top: "max(1rem, env(safe-area-inset-top))",
        right: "calc(var(--inspector-w) + max(1.8rem, env(safe-area-inset-right)))",
        zIndex: 40,
        pointerEvents: "auto",
        filter: "drop-shadow(0 4px 12px rgba(0,0,0,0.3))",
      }}
      aria-label="3D View Cube"
    />
  );
}
