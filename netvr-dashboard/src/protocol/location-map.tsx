export const locationMap = {
  isTracked: 'bool',
  devicePosition: 'vector3',
  deviceRotation: 'quaternion',
  deviceVelocity: 'vector3',
  deviceAngularVelocity: 'vector3',
  deviceAcceleration: 'vector3',
  deviceAngularAcceleration: 'vector3',

  centerEyePosition: 'vector3',
  centerEyeRotation: 'quaternion',
  centerEyeVelocity: 'vector3',
  centerEyeAngularVelocity: 'vector3',
  centerEyeAcceleration: 'vector3',
  centerEyeAngularAcceleration: 'vector3',

  pointerPosition: 'vector3',
  pointerRotation: 'quaternion',
  pointerVelocity: 'vector3',
  pointerAngularVelocity: 'vector3',

  colorCameraRotation: 'quaternion',
  colorCameraAcceleration: 'vector3',
  colorCameraAngularAcceleration: 'vector3',
  colorCameraAngularVelocity: 'vector3',
  colorCameraPosition: 'vector3',
  colorCameraVelocity: 'vector3',
  primary2DAxis: 'vector2',
  secondary2DAxis: 'vector2',
  grip: 'float',
  gripForce: 'float',
  trigger: 'float',
  batteryLevel: 'float',
  secondary2DAxisForce: 'float',
  trackingState: 'uint32',
  gripButton: 'bool',

  menuButton: 'bool',
  menuTouch: 'bool',
  primary2DAxisClick: 'bool',
  primary2DAxisTouch: 'bool',
  primaryButton: 'bool',
  primaryTouch: 'bool',
  secondaryButton: 'bool',
  secondaryTouch: 'bool',
  systemButton: 'bool',
  triggerButton: 'bool',
  triggerTouch: 'bool',
  secondary2DAxisClick: 'bool',
  secondary2DAxisTouch: 'bool',
  userPresence: 'bool',

  leftEyePosition: 'vector3',
  leftEyeRotation: 'quaternion',
  leftEyeVelocity: 'vector3',
  leftEyeAngularVelocity: 'vector3',
  leftEyeAcceleration: 'vector3',
  leftEyeAngularAcceleration: 'vector3',

  rightEyePosition: 'vector3',
  rightEyeRotation: 'quaternion',
  rightEyeVelocity: 'vector3',
  rightEyeAngularVelocity: 'vector3',
  rightEyeAcceleration: 'vector3',
  rightEyeAngularAcceleration: 'vector3',

  //handData: 'hand',
  //eyesData: 'eyes',
} as const