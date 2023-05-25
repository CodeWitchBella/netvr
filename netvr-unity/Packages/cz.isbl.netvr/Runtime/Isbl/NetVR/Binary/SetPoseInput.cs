using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class SetPoseInput: IEquatable<SetPoseInput>, ICloneable {
        public ulong instance;
        public ulong session;
        public uint object_id;
        public Pose pose;

        public SetPoseInput(ulong _instance, ulong _session, uint _object_id, Pose _pose) {
            instance = _instance;
            session = _session;
            object_id = _object_id;
            if (_pose == null) throw new ArgumentNullException(nameof(_pose));
            pose = _pose;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_u64(instance);
            serializer.serialize_u64(session);
            serializer.serialize_u32(object_id);
            pose.Serialize(serializer);
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

        public static SetPoseInput Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            SetPoseInput obj = new SetPoseInput(
            	deserializer.deserialize_u64(),
            	deserializer.deserialize_u64(),
            	deserializer.deserialize_u32(),
            	Pose.Deserialize(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static SetPoseInput BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static SetPoseInput BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            SetPoseInput value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is SetPoseInput other && Equals(other);

        public static bool operator ==(SetPoseInput left, SetPoseInput right) => Equals(left, right);

        public static bool operator !=(SetPoseInput left, SetPoseInput right) => !Equals(left, right);

        public bool Equals(SetPoseInput other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!instance.Equals(other.instance)) return false;
            if (!session.Equals(other.session)) return false;
            if (!object_id.Equals(other.object_id)) return false;
            if (!pose.Equals(other.pose)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + instance.GetHashCode();
                value = 31 * value + session.GetHashCode();
                value = 31 * value + object_id.GetHashCode();
                value = 31 * value + pose.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public SetPoseInput Clone() => (SetPoseInput)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
