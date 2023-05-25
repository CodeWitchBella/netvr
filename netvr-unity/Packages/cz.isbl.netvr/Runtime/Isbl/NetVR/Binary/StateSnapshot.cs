using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class StateSnapshot: IEquatable<StateSnapshot>, ICloneable {
        public Serde.ValueArray<Controller> controllers;
        public Pose view;
        public uint required_configuration;

        public StateSnapshot(Serde.ValueArray<Controller> _controllers, Pose _view, uint _required_configuration) {
            if (_controllers == null) throw new ArgumentNullException(nameof(_controllers));
            controllers = _controllers;
            if (_view == null) throw new ArgumentNullException(nameof(_view));
            view = _view;
            required_configuration = _required_configuration;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            TraitHelpers.serialize_vector_Controller(controllers, serializer);
            view.Serialize(serializer);
            serializer.serialize_u32(required_configuration);
            serializer.decrease_container_depth();
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static StateSnapshot Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            StateSnapshot obj = new StateSnapshot(
            	TraitHelpers.deserialize_vector_Controller(deserializer),
            	Pose.Deserialize(deserializer),
            	deserializer.deserialize_u32());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static StateSnapshot BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static StateSnapshot BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            StateSnapshot value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is StateSnapshot other && Equals(other);

        public static bool operator ==(StateSnapshot left, StateSnapshot right) => Equals(left, right);

        public static bool operator !=(StateSnapshot left, StateSnapshot right) => !Equals(left, right);

        public bool Equals(StateSnapshot other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!controllers.Equals(other.controllers)) return false;
            if (!view.Equals(other.view)) return false;
            if (!required_configuration.Equals(other.required_configuration)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + controllers.GetHashCode();
                value = 31 * value + view.GetHashCode();
                value = 31 * value + required_configuration.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public StateSnapshot Clone() => (StateSnapshot)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
